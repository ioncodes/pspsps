import { LazyLoadImage } from 'react-lazy-load-image-component'
import 'react-lazy-load-image-component/src/effects/blur.css'
import { Box, Typography, Chip, Card } from '@mui/material'

function GameGallery({ game }) {
  const getImagePath = (filename) => {
    return `/screenshots/${game.name}/${filename}`
  }

  return (
    <Box component="section" sx={{ mb: 8 }}>
      <Box sx={{ mb: 3, display: 'flex', alignItems: 'center', gap: 2 }}>
        <Typography variant="h2" sx={{ lineHeight: 1, mb: 0 }}>
          {game.name}
        </Typography>
        <Chip
          size="small"
          label={`${game.images.length} screenshot${game.images.length !== 1 ? 's' : ''}`}
          sx={{
            background: 'linear-gradient(45deg, #ef4444 30%, #f97316 90%)',
            color: 'white',
            fontWeight: 600,
          }}
        />
      </Box>

      <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 2 }}>
        {game.images.map((filename) => {
          const frameNumber = filename.replace('.png', '')
          return (
            <Card
              key={filename}
              elevation={2}
              sx={{
                overflow: 'hidden',
                transition: 'all 0.3s ease',
                lineHeight: 0,
                '&:hover': {
                  transform: 'scale(1.05)',
                  boxShadow: (theme) => `0 8px 24px ${theme.palette.primary.main}40`,
                },
              }}
            >
              <LazyLoadImage
                src={getImagePath(filename)}
                alt={`${game.name} - Frame ${frameNumber}`}
                effect="blur"
                style={{
                  maxHeight: '240px',
                  display: 'block',
                  height: 'auto',
                  verticalAlign: 'bottom',
                }}
                placeholderSrc="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 400 300'%3E%3Crect fill='%2334495e' width='400' height='300'/%3E%3C/svg%3E"
              />
            </Card>
          )
        })}
      </Box>
    </Box>
  )
}

export default GameGallery
