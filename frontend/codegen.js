import { TSBuilder } from '@cosmwasm/ts-codegen';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

const builder = new TSBuilder({
  contracts: [
    {
      name: 'TgContractPayments',
      dir: resolve(__dirname, '../builds/schema/payments')
    }
  ],
  outPath: resolve(__dirname, './src/contracts'),
  options: {
    bundle: {
      bundleFile: 'index.ts',
      scope: 'contracts'
    },
    types: {
      enabled: true
    },
    client: {
      enabled: true
    },
    reactQuery: {
      enabled: false,
      optionalClient: true,
      version: 'v4',
      mutations: true,
      queryKeys: true,
      queryFactory: true
    },
    messageComposer: {
      enabled: true
    },
    messageBuilder: {
      enabled: false
    }
  }
});

await builder.build();
