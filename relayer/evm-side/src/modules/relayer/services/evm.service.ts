import { Inject, Injectable, Logger } from '@nestjs/common';
import { Contract, Interface, Provider, Wallet } from 'ethers';
import {
  EVM_RPCS,
  getEvmFinalizationBlockGapEnvVarKey,
  getEvmWithdrawContractEnvVarKey,
  getEvmWithdrawContractOwnerPkEnvVarKey,
  MainConfigService,
} from 'src/config';
import { ExitModel } from 'src/models/exit.model';
import { WITHDRAW_CONTRACT_ABI } from '../evm.abi';

@Injectable()
export class EVMRelayerService {
  private readonly logger = new Logger(EVMRelayerService.name);

  constructor(
    @Inject(EVM_RPCS)
    private readonly rpcs: Map<bigint, Provider>,
    private readonly config: MainConfigService,
  ) {}

  async relay(exit: ExitModel) {
    const rpc = this.rpcs.get(exit.to.chainId);
    const withdrawContract = this.config.getEvmChainConf(
      getEvmWithdrawContractEnvVarKey(exit.to.chainId),
    );
    const privateKey = this.config.getEvmChainConf(
      getEvmWithdrawContractOwnerPkEnvVarKey(exit.to.chainId),
    );
    const finalizationBlockGap = Number.parseInt(
      this.config.getEvmChainConf(
        getEvmFinalizationBlockGapEnvVarKey(exit.to.chainId),
      ) || '0',
    );
    if (rpc == null || withdrawContract == null || privateKey == null) {
      throw new Error(`Unknown EVM chain with id ${exit.to.chainId}`);
    }
    const evmClient = new Wallet(privateKey, rpc);
    const contract = new Contract(
      withdrawContract,
      new Interface(WITHDRAW_CONTRACT_ABI),
      evmClient,
    );
    const tx = await contract.issueToken(
      exit.receiver,
      exit.assetAmount.toFixed(),
      exit.assetOrigin.chainId.toString(10),
      exit.assetAddress,
      exit.assetName || `${exit.assetSymbol} wrapped`,
      exit.assetSymbol,
      exit.assetDecimals,
    );
    this.logger.debug(`Transaction sent with hash ${tx.hash}`);
    await tx.wait(Math.max(finalizationBlockGap, 1));
    this.logger.debug(`Transaction verified with hash ${tx.hash}`);
  }
}
