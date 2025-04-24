export const WITHDRAW_CONTRACT_ABI = [
  {
    type: 'function',
    name: 'issueToken',
    constant: false,
    anonymous: false,
    stateMutability: 'nonpayable',
    inputs: [
      {
        name: 'receiver',
        type: 'address',
        storage_location: 'default',
        simple_type: { type: 'address' },
      },
      {
        name: 'amount',
        type: 'uint256',
        storage_location: 'default',
        simple_type: { type: 'uint' },
      },
      {
        name: 'originTokenNetwork',
        type: 'uint32',
        storage_location: 'default',
        simple_type: { type: 'uint' },
      },
      {
        name: 'originTokenAddress',
        type: 'address',
        storage_location: 'default',
        simple_type: { type: 'address' },
      },
      {
        name: 'tokenName',
        type: 'string',
        storage_location: 'default',
        simple_type: { type: 'string' },
      },
      {
        name: 'tokenSymbol',
        type: 'string',
        storage_location: 'default',
        simple_type: { type: 'string' },
      },
      {
        name: 'tokenDecimals',
        type: 'uint8',
        storage_location: 'default',
        simple_type: { type: 'uint' },
      },
    ],
    outputs: [],
  },
] as const;
