import { copyFile, mkdir, readdir, rm } from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'

function readArgument(name) {
  const index = process.argv.indexOf(name)
  return index >= 0 ? process.argv[index + 1] : undefined
}

const target = readArgument('--target')
const artifactName = readArgument('--name')

if (!target || !artifactName) {
  throw new Error('Usage: node scripts/collect-tauri-installers.mjs --target <rust-target> --name <artifact-name>')
}

const bundleDirectory = path.join('src-tauri', 'target', target, 'release', 'bundle')
const outputDirectory = 'release-assets'
const installerSuffixes = ['.app.tar.gz', '.dmg', '.msi', '.exe']
const installers = []

async function collectFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true })
  for (const entry of entries) {
    const filePath = path.join(directory, entry.name)
    if (entry.isDirectory()) {
      if (!entry.name.endsWith('.app')) await collectFiles(filePath)
      continue
    }
    if (entry.isFile() && installerSuffixes.some((suffix) => entry.name.toLowerCase().endsWith(suffix))) {
      installers.push(filePath)
    }
  }
}

await rm(outputDirectory, { recursive: true, force: true })
await mkdir(outputDirectory, { recursive: true })
await collectFiles(bundleDirectory)

if (installers.length === 0) {
  throw new Error(`No installer files found in ${bundleDirectory}`)
}

for (const installer of installers) {
  const destination = path.join(outputDirectory, `nimbo-${artifactName}-${path.basename(installer)}`)
  await copyFile(installer, destination)
  console.log(`Prepared ${destination}`)
}
