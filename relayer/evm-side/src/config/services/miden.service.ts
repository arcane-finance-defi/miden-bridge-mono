import { Injectable, Logger } from '@nestjs/common';
import type { paths, components } from 'api';
import { createPathBasedClient, PathBasedClient } from 'openapi-fetch';
import { SendRequest, SendResponse } from '../interfaces';

@Injectable()
export class MidenApiService {
  private readonly client: PathBasedClient<paths, 'application/json'>;
  private readonly logger = new Logger(MidenApiService.name);

  constructor(apiUrl: string) {
    this.client = createPathBasedClient<paths, 'application/json'>({
      baseUrl: apiUrl,
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

    if (status !== 200 && status !== 201) {
      this.logger.warn(`/mint Api responded with status ${status}`);
      throw new Error(
        `Miden module responds with error: ${error.code} ${error.message}`,
      );
    }

    if (response == null) {
      throw new Error(`Malformed response from miden module: ${response}`);
    }

    return response;
  }

  async pollExits(
    fromHeight: number,
  ): Promise<components['schemas']['PolledEvents']> {
    const {
      data: response,
      response: { status },
      error,
    } = await this.client['/poll'].GET({
      params: { query: { fromHeight } },
    });

    if (status !== 200) {
      this.logger.warn(`/poll Api responded with status ${status}`);
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
