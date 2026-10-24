use chrono::{DateTime, Utc};
use diesel::{
  ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper,
  dsl::{insert_into, update},
};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use flow::{refresh_tokens, sessions, user::User, users};
use uuid::Uuid;

use super::models::{RefreshToken, Session};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshRotationOutcome {
  Rotated,
  Reused,
  Invalid,
  SessionRevoked,
}

pub struct AuthRepository;

impl AuthRepository {
  pub async fn upsert_user(
    connection: &mut AsyncPgConnection,
    email: &str,
  ) -> Result<User, diesel::result::Error> {
    insert_into(users::table)
      .values(users::uid.eq(Some(email)))
      .on_conflict(users::uid)
      .do_update()
      .set(users::uid.eq(users::uid))
      .returning(User::as_returning())
      .get_result(connection)
      .await
  }

  pub async fn create_session_with_refresh_token(
    connection: &mut AsyncPgConnection,
    new_session: Session,
    new_refresh_token: RefreshToken,
  ) -> Result<(Session, RefreshToken), diesel::result::Error> {
    connection
      .transaction(async |conn| {
        let session = insert_into(sessions::table)
          .values(new_session)
          .returning(Session::as_returning())
          .get_result(conn)
          .await?;

        let refresh_token = insert_into(refresh_tokens::table)
          .values(new_refresh_token)
          .returning(RefreshToken::as_returning())
          .get_result(conn)
          .await?;

        Ok((session, refresh_token))
      })
      .await
  }

  pub async fn find_refresh_token_by_hash(
    connection: &mut AsyncPgConnection,
    token_bytes: &[u8],
  ) -> Result<RefreshToken, diesel::result::Error> {
    refresh_tokens::table
      .filter(refresh_tokens::token.eq(token_bytes))
      .select(RefreshToken::as_select())
      .first(connection)
      .await
  }

  pub async fn find_session_by_id(
    connection: &mut AsyncPgConnection,
    session_id: Uuid,
  ) -> Result<Session, diesel::result::Error> {
    sessions::table
      .filter(sessions::id.eq(session_id))
      .select(Session::as_select())
      .first(connection)
      .await
  }

  pub async fn find_sessions_by_user(
    connection: &mut AsyncPgConnection,
    user_id: Uuid,
  ) -> Result<Vec<Session>, diesel::result::Error> {
    sessions::table
      .filter(sessions::user.eq(user_id))
      .filter(sessions::revokedAt.is_null())
      .order(sessions::createdAt.desc())
      .select(Session::as_select())
      .load(connection)
      .await
  }

  pub async fn rotate_refresh_token(
    connection: &mut AsyncPgConnection,
    hash: &[u8],
    now: DateTime<Utc>,
    replacement: RefreshToken,
  ) -> Result<RefreshRotationOutcome, diesel::result::Error> {
    let hash = hash.to_vec();

    connection
      .transaction(async |connection| {
        let Some(stored) = refresh_tokens::table
          .filter(refresh_tokens::token.eq(&hash))
          .for_update()
          .first::<RefreshToken>(connection)
          .await
          .optional()?
        else {
          return Ok(RefreshRotationOutcome::Invalid);
        };

        if stored.consumed_at.is_some() {
          update(sessions::table.filter(sessions::id.eq(stored.session)))
            .set(sessions::revokedAt.eq(Some(now)))
            .execute(connection)
            .await?;

          update(refresh_tokens::table.filter(refresh_tokens::id.eq(stored.session)))
            .set(refresh_tokens::consumedAt.eq(Some(now)))
            .execute(connection)
            .await?;

          return Ok(RefreshRotationOutcome::Reused);
        }

        if stored.revoked_at.is_some() || stored.expires_at <= now {
          return Ok(RefreshRotationOutcome::Invalid);
        }

        let Some(session) = sessions::table
          .filter(sessions::id.eq(stored.session))
          .for_update()
          .first::<Session>(connection)
          .await
          .optional()?
        else {
          return Ok(RefreshRotationOutcome::SessionRevoked);
        };

        if session.revoked_at.is_some() || session.expires_at <= now {
          diesel::update(refresh_tokens::table.filter(refresh_tokens::session.eq(session.id)))
            .set(refresh_tokens::revokedAt.eq(Some(now)))
            .execute(connection)
            .await?;

          return Ok(RefreshRotationOutcome::SessionRevoked);
        }

        if replacement.session != session.id {
          return Ok(RefreshRotationOutcome::Invalid);
        }

        let updated = diesel::update(
          refresh_tokens::table
            .filter(refresh_tokens::id.eq(stored.id))
            .filter(refresh_tokens::consumedAt.is_null())
            .filter(refresh_tokens::revokedAt.is_null()),
        )
        .set(refresh_tokens::consumedAt.eq(Some(now)))
        .execute(connection)
        .await?;

        if updated != 1 {
          return Ok(RefreshRotationOutcome::Invalid);
        }

        diesel::insert_into(refresh_tokens::table)
          .values(&replacement)
          .execute(connection)
          .await?;

        Ok(RefreshRotationOutcome::Rotated)
      })
      .await
  }

  pub async fn revoke_session_and_tokens(
    connection: &mut AsyncPgConnection,
    session_id: Uuid,
    user_id: Uuid,
    now: DateTime<Utc>,
  ) -> Result<Session, diesel::result::Error> {
    connection
      .transaction(async |conn| {
        let session = update(
          sessions::table
            .filter(sessions::id.eq(session_id))
            .filter(sessions::user.eq(user_id)),
        )
        .set(sessions::revokedAt.eq(Some(now)))
        .returning(Session::as_returning())
        .get_result(conn)
        .await?;

        update(refresh_tokens::table.filter(refresh_tokens::session.eq(session_id)))
          .set(refresh_tokens::revokedAt.eq(Some(now)))
          .execute(conn)
          .await?;

        Ok(session)
      })
      .await
  }
}
