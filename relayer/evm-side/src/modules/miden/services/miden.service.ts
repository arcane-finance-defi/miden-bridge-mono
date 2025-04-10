import { Injectable } from '@nestjs/common';
import type { paths } from 'api';
import { createPathBasedClient, PathBasedClient } from 'openapi-fetch';
import { MainConfigService } from 'src/config';
import { SendRequest, SendResponse } from '../interfaces';

@Injectable()
export class MidenApiService {
  private readonly client: PathBasedClient<paths, 'application/json'>;

  constructor(config: MainConfigService) {
    this.client = createPathBasedClient<paths, 'application/json'>({
      baseUrl: config.getEnv('MIDEN_API_URL'),
    });
  }

  async send({ amount, asset, recipient }: SendRequest): Promise<SendResponse> {
    const {
      data: response,
      response: { status },
      error,
    } = await this.client['/mint'].POST({
      body: {
        asset: {
          assetSymbol: asset.symbol,
          decimals: asset.decimals,
          originAddress: asset.address,
          originNetwork: asset.network,
        },
        recipient,
        amount: amount.toNumber(),
      },
    });

    if (status !== 200) {
      throw new Error(
        `Miden module responds with error: ${error.code} ${error.message}`,
      );
    }

    if (response == null) {
      throw new Error(`Malformed response from miden module: ${response}`);
    }

    return response;
  }
}
