import { readFile } from 'node:fs/promises'
import process from 'node:process'

const tag = process.env.RELEASE_TAG
if (!tag?.startsWith('v')) {
  throw new Error('RELEASE_TAG must be a version tag beginning with v, for example v0.1.0')
}

const expectedVersion = tag.slice(1)
const packageJson = JSON.parse(await readFile('package.json', 'utf8'))
const tauriConfig = JSON.parse(await readFile('src-tauri/tauri.conf.json', 'utf8'))
const cargoManifest = await readFile('src-tauri/Cargo.toml', 'utf8')
const cargoVersion = cargoManifest.match(/^version\s*=\s*"([^"]+)"/m)?.[1]
const versions = {
  'package.json': packageJson.version,
  'src-tauri/tauri.conf.json': tauriConfig.version,
  'src-tauri/Cargo.toml': cargoVersion,
}

const mismatches = Object.entries(versions)
  .filter(([, version]) => version !== expectedVersion)
  .map(([file, version]) => `${file}=${version ?? 'missing'}`)

if (mismatches.length > 0) {
  throw new Error(`Tag ${tag} does not match app version ${expectedVersion}; found ${mismatches.join(', ')}`)
}

console.log(`Release tag ${tag} matches all application version files.`)
