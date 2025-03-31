import Joi from '@hapi/joi';

export interface IEnvConfig {
  MIDEN_CHAIN_IDS: string;
  EVM_CHAIN_IDS: string;
}

export const EnvConfigSchema = Joi.object<IEnvConfig>({
  MIDEN_CHAIN_IDS: Joi.string()
    .regex(/\d+(,\d+)*/)
    .required(),
  EVM_CHAIN_IDS: Joi.string()
    .regex(/\d+(,\d+)*/)
    .required(),
}).options({
  abortEarly: false,
});
