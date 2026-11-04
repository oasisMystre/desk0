use diesel::{
  BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper,
  dsl::{insert_into, max},
};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use flow::{
  agent::{Agent, AgentAsset},
  agent_assets, agents, organizations,
};

pub struct AgentRepository;

impl AgentRepository {
  pub async fn allocate_and_create_agent(
    connection: &mut AsyncPgConnection,
    organization: uuid::Uuid,
  ) -> Result<Agent, diesel::result::Error> {
    connection
      .transaction(async |conn| {
        organizations::table
          .filter(organizations::id.eq(organization))
          .for_update()
          .execute(conn)
          .await?;

        let version = agents::table
          .filter(agents::organization.eq(organization))
          .select(max(agents::version))
          .first::<Option<i32>>(conn)
          .await?;
        let version = version.unwrap_or(0) + 1;

        insert_into(agents::table)
          .values(((
            agents::organization.eq(organization),
            agents::version.eq(version),
          ),))
          .get_result::<Agent>(conn)
          .await
      })
      .await
  }

  pub async fn find_agents_by_organization(
    connection: &mut AsyncPgConnection,
    organization: uuid::Uuid,
  ) -> Result<Vec<(Agent, AgentAsset)>, diesel::result::Error> {
    agents::table
      .inner_join(agent_assets::table.on(agent_assets::agent.eq(agents::id)))
      .filter(agents::organization.eq(organization))
      .select((Agent::as_select(), AgentAsset::as_select()))
      .load::<(Agent, AgentAsset)>(connection)
      .await
  }

  pub async fn find_agent_assets_by_agent_and_organization(
    connection: &mut AsyncPgConnection,
    agent: uuid::Uuid,
    organization: uuid::Uuid,
  ) -> Result<Vec<AgentAsset>, diesel::result::Error> {
    agent_assets::table
      .inner_join(agents::table.on(agents::id.eq(agent_assets::agent)))
      .filter(
        agent_assets::agent
          .eq(agent)
          .and(agents::organization.eq(organization)),
      )
      .select(AgentAsset::as_select())
      .load::<AgentAsset>(connection)
      .await
  }
}
