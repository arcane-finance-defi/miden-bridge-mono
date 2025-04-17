import { Inject, Injectable, Logger } from '@nestjs/common';
import { ZeroAddress } from 'ethers';
import { MIDEN_RPCS, MidenApiService } from 'src/config';
import { ExitModel } from 'src/models/exit.model';

@Injectable()
export class MidenRelayerService {
  private readonly logger = new Logger(MidenRelayerService.name);

  constructor(
    @Inject(MIDEN_RPCS)
    private readonly midenRpcs: Map<bigint, MidenApiService>,
  ) {}

  async relay(exit: ExitModel) {
    const rpc = this.midenRpcs.get(exit.to.chainId);
    if (rpc == null) {
      throw new Error(`Unknown miden chain with chainId: ${exit.to.chainId}`);
    }
    const isWethAsset =
      exit.assetOrigin.chainId === 0n && exit.assetAddress === ZeroAddress;
    this.logger.debug(
      `Exit with id ${exit.id} is about to relay to miden chain`,
    );
    const response = await rpc.send({
      amount: exit.assetAmount,
      asset: {
        address: exit.assetAddress,
        decimals: isWethAsset ? 18 : exit.assetDecimals,
        network: Number(exit.assetOrigin.chainId),
        symbol: isWethAsset ? 'WETH' : exit.assetSymbol,
      },
      recipient: exit.calldata,
    });

    this.logger.debug(
      `Deposit relayed to the miden with note id: ${response.noteId}`,
    );
  }
}
