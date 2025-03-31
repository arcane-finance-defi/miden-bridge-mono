import { Interface } from 'ethers';

export const BRIDGE_EVENT_ABI = new Interface([
  {
    type: 'event',
    name: 'BridgeEvent',
    constant: false,
    anonymous: false,
    stateMutability: '',
    inputs: [
      {
        name: 'leafType',
        type: 'uint8',
        indexed: false,
      },
      {
        name: 'originNetwork',
        type: 'uint32',
        indexed: false,
      },
      {
        name: 'originAddress',
        type: 'address',
        indexed: false,
      },
      {
        name: 'destinationNetwork',
        type: 'uint32',
        indexed: false,
      },
      {
        name: 'destinationAddress',
        type: 'address',
        indexed: false,
      },
      {
        name: 'amount',
        type: 'uint256',
        indexed: false,
      },
      {
        name: 'metadata',
        type: 'bytes',
        indexed: false,
      },
      {
        name: 'depositCount',
        type: 'uint32',
        indexed: false,
      },
    ],
    outputs: null,
  },
]);
