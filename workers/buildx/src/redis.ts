import { Redis, type RedisOptions } from "ioredis";

import { getEnv } from "./env";

type Option = {
  port: number;
  host: string;
} & RedisOptions;

function _createRedis(url: string, options: RedisOptions): Redis;
function _createRedis(options: Option): Redis;
function _createRedis(option: string | Option, others?: RedisOptions) {
  if (typeof option === "object") {
    const { host, port, ...options } = option;
    return new Redis({
      name: option.name,
      sentinelUsername: option.username,
      sentinelPassword: option.password,
      sentinels: [
        {
          host: host,
          port: port,
        },
      ],
      ...options,
    });
  }

  // @ts-expect-error don't check type here
  return new Redis(option, others);
}

export const createRedis = (options?: RedisOptions) => {
  let redis: Redis;

  if (
    "APP_REDIS_MASTER_NAME" in process.env &&
    "APP_REDIS_SENTINEL_PORT" in process.env &&
    "APP_REDIS_SENTINEL_HOSTNAME" in process.env &&
    "APP_REDIS_PASSWORD" in process.env
  )
    redis = _createRedis({
      username: getEnv("REDIS_USERNAME"),
      name: getEnv("REDIS_MASTER_NAME"),
      port: getEnv("REDIS_SENTINEL_PORT", Number),
      host: getEnv("REDIS_SENTINEL_HOSTNAME"),
      password: getEnv("REDIS_PASSWORD", true),
      ...options,
    });
  else if (options)
    redis = _createRedis(getEnv("REDIS_URL"), {
      password: getEnv("REDIS_PASSWORD", true),
      ...options,
    });
  else redis = _createRedis(getEnv("REDIS_URL"));

  return redis;
};
