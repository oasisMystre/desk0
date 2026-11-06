import { format } from "util";

export type LogFunction = (row: string) => Promise<number>;

export type Logger = {
  info: typeof console.log;
  error: typeof console.error;
};

export const createJobLogger = (fn?: LogFunction): Logger => {
  const createLogger =
    (level: keyof Logger): LogFunction =>
    async (message: string, ...args) => {
      console[level](format("[%s] %s", level, message), ...args);

      if (fn)
        return fn(message).catch((error) => {
          console.error("Failed to append BullMQ log", error);
          return -1;
        });

      return -1;
    };

  return {
    info: createLogger("info"),
    error: createLogger("error"),
  };
};
