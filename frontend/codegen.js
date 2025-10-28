import { TSBuilder } from '@cosmwasm/ts-codegen';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import { readdir, readFile, writeFile } from 'fs/promises';
import { join } from 'path';
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

// Post-process generated files to add @ts-nocheck for strict TypeScript configs
const contractsDir = resolve(__dirname, './src/contracts');
const files = await readdir(contractsDir);

for (const file of files) {
  if (file.endsWith('.ts')) {
    const filePath = join(contractsDir, file);
    const content = await readFile(filePath, 'utf-8');
    
    // Check if @ts-nocheck already exists
    if (!content.includes('@ts-nocheck')) {
      // Add @ts-nocheck after the initial comment block
      const lines = content.split('\n');
      let insertIndex = 0;
      
      // Find the end of the generated comment block
      for (let i = 0; i < lines.length; i++) {
        if (lines[i].startsWith('*/')) {
          insertIndex = i + 1;
          break;
        }
      }
      
      // Insert @ts-nocheck directive
      lines.splice(insertIndex, 0, '// @ts-nocheck');
      
      await writeFile(filePath, lines.join('\n'), 'utf-8');
    }
  }
}

console.log('✨ TypeScript bindings generated successfully!');
