import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'
import fs from 'fs'

// Plugin to generate images manifest and serve screenshots
function generateImagesManifest() {
  return {
    name: 'generate-images-manifest',
    configureServer(server) {
      // Generate manifest on server start
      generateManifest()

      // Add middleware to serve screenshots
      server.middlewares.use('/screenshots', (req, res, next) => {
        const screenshotsPath = path.resolve(__dirname, '../screenshots')
        // Decode URL to handle spaces and special characters
        const decodedUrl = decodeURIComponent(req.url)
        const filePath = path.join(screenshotsPath, decodedUrl)

        console.log('Screenshot request:', decodedUrl)
        console.log('File path:', filePath)
        console.log('Exists:', fs.existsSync(filePath))

        if (fs.existsSync(filePath) && fs.statSync(filePath).isFile()) {
          res.setHeader('Content-Type', 'image/png')
          res.setHeader('Cache-Control', 'public, max-age=31536000')
          fs.createReadStream(filePath).pipe(res)
        } else {
          console.log('File not found:', filePath)
          next()
        }
      })

      // Watch for changes in screenshots directory
      const screenshotsPath = path.resolve(__dirname, '../screenshots')
      if (fs.existsSync(screenshotsPath)) {
        server.watcher.add(screenshotsPath)
        server.watcher.on('change', (file) => {
          if (file.startsWith(screenshotsPath)) {
            generateManifest()
          }
        })
      }
    },
    buildStart() {
      generateManifest()
    }
  }
}

function generateManifest() {
  const screenshotsPath = path.resolve(__dirname, '../screenshots')
  const publicPath = path.resolve(__dirname, 'public')

  if (!fs.existsSync(screenshotsPath)) {
    console.log('Screenshots directory not found, creating empty manifest')
    fs.writeFileSync(
      path.join(publicPath, 'images.json'),
      JSON.stringify({ games: [] }, null, 2)
    )
    return
  }

  const games = []
  const gameDirs = fs.readdirSync(screenshotsPath)

  for (const gameDir of gameDirs) {
    const gamePath = path.join(screenshotsPath, gameDir)
    const stat = fs.statSync(gamePath)

    if (!stat.isDirectory()) continue

    const files = fs.readdirSync(gamePath)
      .filter(file => file.endsWith('.png'))
      .sort((a, b) => {
        // Sort numerically by frame number
        const numA = parseInt(a.replace('.png', ''))
        const numB = parseInt(b.replace('.png', ''))
        return numA - numB
      })

    if (files.length > 0) {
      games.push({
        name: gameDir,
        images: files
      })
    }
  }

  // Sort games alphabetically
  games.sort((a, b) => a.name.localeCompare(b.name))

  const manifest = { games }

  if (!fs.existsSync(publicPath)) {
    fs.mkdirSync(publicPath, { recursive: true })
  }

  fs.writeFileSync(
    path.join(publicPath, 'images.json'),
    JSON.stringify(manifest, null, 2)
  )

  console.log(`Generated manifest with ${games.length} game(s)`)
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), generateImagesManifest()],
  server: {
    fs: {
      // Allow serving files from parent directory
      allow: ['..']
    }
  },
  resolve: {
    alias: {
      '@screenshots': path.resolve(__dirname, '../screenshots')
    }
  }
})
