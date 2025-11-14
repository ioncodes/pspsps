import { useState, useEffect } from 'react'
import {
  AppBar,
  Box,
  Container,
  TextField,
  Typography,
  CircularProgress,
  Alert,
  InputAdornment,
  IconButton,
  Toolbar,
  Chip,
} from '@mui/material'
import SearchIcon from '@mui/icons-material/Search'
import ClearIcon from '@mui/icons-material/Clear'
import CameraAltIcon from '@mui/icons-material/CameraAlt'
import GameGallery from './components/GameGallery'
import PS1Logo from './components/PS1Logo'

function App() {
  const [games, setGames] = useState([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)
  const [searchQuery, setSearchQuery] = useState('')

  useEffect(() => {
    fetch('/images.json')
      .then(res => res.json())
      .then(data => {
        setGames(data.games)
        setLoading(false)
      })
      .catch(err => {
        setError(err.message)
        setLoading(false)
      })
  }, [])

  const filteredGames = games.filter(game =>
    game.name.toLowerCase().includes(searchQuery.toLowerCase())
  )

  const totalScreenshots = games.reduce((sum, g) => sum + g.images.length, 0)

  if (loading) {
    return (
      <Box
        sx={{
          minHeight: '100vh',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          bgcolor: 'background.default',
        }}
      >
        <Box textAlign="center">
          <CircularProgress size={60} />
          <Typography variant="h6" sx={{ mt: 2, color: 'text.secondary' }}>
            Loading screenshots...
          </Typography>
        </Box>
      </Box>
    )
  }

  if (error) {
    return (
      <Box
        sx={{
          minHeight: '100vh',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          bgcolor: 'background.default',
          p: 3,
        }}
      >
        <Alert severity="error" sx={{ maxWidth: 600 }}>
          Error: {error}
        </Alert>
      </Box>
    )
  }

  if (games.length === 0) {
    return (
      <Box
        sx={{
          minHeight: '100vh',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          bgcolor: 'background.default',
        }}
      >
        <Box textAlign="center">
          <CameraAltIcon sx={{ fontSize: 80, color: 'text.secondary', mb: 2 }} />
          <Typography variant="h5" color="text.primary" gutterBottom>
            No screenshots found
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Run rom-db to generate some!
          </Typography>
        </Box>
      </Box>
    )
  }

  return (
    <Box sx={{ minHeight: '100vh', bgcolor: 'background.default' }}>
      <AppBar position="sticky" elevation={0} sx={{ borderBottom: 1, borderColor: 'divider' }}>
        <Toolbar sx={{ py: 2 }}>
          <Container maxWidth="xl">
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 3, flexWrap: { xs: 'wrap', md: 'nowrap' } }}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, flexShrink: 0 }}>
                <PS1Logo sx={{ fontSize: 56, color: 'primary.main' }} />
                <Box>
                  <Typography
                    variant="h1"
                    sx={{
                      fontSize: { xs: '1.5rem', sm: '2rem', md: '2.25rem' },
                      lineHeight: 1.3,
                      pb: 0.5,
                      background: 'linear-gradient(45deg, #ef4444 30%, #f97316 90%)',
                      backgroundClip: 'text',
                      WebkitBackgroundClip: 'text',
                      WebkitTextFillColor: 'transparent',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    pspsps screenshot gallery
                  </Typography>
                  <Box sx={{ display: 'flex', gap: 1, mt: 1.5 }}>
                    <Chip
                      size="small"
                      label={`${games.length} game${games.length !== 1 ? 's' : ''}`}
                      sx={{
                        background: 'linear-gradient(45deg, #ef4444 30%, #f97316 90%)',
                        color: 'white',
                        fontWeight: 600,
                      }}
                    />
                    <Chip
                      size="small"
                      label={`${totalScreenshots} screenshots`}
                      sx={{
                        background: 'linear-gradient(45deg, #ef4444 30%, #f97316 90%)',
                        color: 'white',
                        fontWeight: 600,
                      }}
                    />
                  </Box>
                </Box>
              </Box>

              <TextField
                fullWidth
                variant="outlined"
                placeholder="Search games..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                InputProps={{
                  startAdornment: (
                    <InputAdornment position="start">
                      <SearchIcon />
                    </InputAdornment>
                  ),
                  endAdornment: searchQuery && (
                    <InputAdornment position="end">
                      <IconButton size="small" onClick={() => setSearchQuery('')}>
                        <ClearIcon />
                      </IconButton>
                    </InputAdornment>
                  ),
                }}
              />
            </Box>
          </Container>
        </Toolbar>
      </AppBar>

      <Container maxWidth="xl" sx={{ py: 6 }}>
        {filteredGames.length === 0 ? (
          <Box textAlign="center" py={8}>
            <SearchIcon sx={{ fontSize: 80, color: 'text.secondary', mb: 2 }} />
            <Typography variant="h5" color="text.secondary">
              No games found matching "{searchQuery}"
            </Typography>
          </Box>
        ) : (
          filteredGames.map((game) => (
            <GameGallery key={game.name} game={game} />
          ))
        )}
      </Container>

      <Box
        component="footer"
        sx={{
          borderTop: 1,
          borderColor: 'divider',
          bgcolor: 'background.paper',
          py: 3,
          mt: 8,
        }}
      >
        <Container maxWidth="xl">
          <Typography variant="body2" color="text.secondary" align="center">
            Generated by <strong>rom-db</strong>
          </Typography>
        </Container>
      </Box>
    </Box>
  )
}

export default App
