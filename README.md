# TWO PEOPLE

**A fast-paced "Which type are you?" card game!**

> CHOOSE FAST! There are only 2 types of people in the world...

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Bevy](https://img.shields.io/badge/bevy-232326?style=for-the-badge&logo=bevy&logoColor=white)

## About

Two People is a vibrant, meme-y, TikTok-style card game where you have 5 seconds to choose between two personality types. At the end, discover if you're a **Chaotic Gremlin** or a **Functioning Adult**!

Inspired by viral "Which type are you?" videos and the YouTube show "Cut".

## Features

- **5-second timer** - Choose fast or the game picks for you!
- **Vibrant colors** - Electric pink, electric blue, punchy animations
- **Satisfying interactions** - Hover effects, screen shake, bouncy cards
- **5 personality questions** - Each more relatable than the last
- **MySQL score tracking** - Save your results (optional)
- **Cross-platform** - Works on macOS and Windows

## Screenshots

The game features:
- Floating particles in the background
- Cards that bob gently and scale on hover
- A pulsing countdown timer that turns red when time is running out
- Screen shake on card selection

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- Optional: MySQL 8.0+ for score tracking

### Run the Game

```bash
# Clone the repo
git clone https://github.com/your-username/two-people.git
cd two-people

# Run in development mode
cargo run

# Or build optimized release
cargo build --release
./target/release/two-people
```

### Database Setup (Optional)

1. Create a MySQL database:
```sql
CREATE DATABASE two_people;
```

2. Run the migrations:
```bash
mysql -u root -p two_people < migrations/001_init.sql
```

3. Set environment variable:
```bash
# Create .env file
echo 'DATABASE_URL=mysql://root:your_password@localhost:3306/two_people' > .env
```

## How to Play

1. **Watch the intro** - "TWO PEOPLE" flashes on screen
2. **Read the question** - "There are 2 types of people..."
3. **Click a card** - Left (pink) or Right (blue)
4. **Beat the timer!** - 5 seconds per question
5. **See your result** - Are you a Gremlin or a Functioning Adult?
6. **Press R** - Play again!

## The Questions

1. **Early Bird** vs **Night Owl**
2. **Snooze x100** vs **Up & At 'Em**
3. **5% Battery** vs **Always 100%**
4. **3-5 Days** vs **Instant Reply**
5. **Couch + Netflix** vs **Out Till 4AM**

## Tech Stack

- **Game Engine**: [Bevy 0.15](https://bevyengine.org/)
- **Language**: Rust
- **Database**: MySQL (via sqlx)
- **Audio**: Bevy Audio (planned)

## Project Structure

```
two-people/
├── src/
│   └── main.rs          # Complete game implementation
├── assets/
│   └── sounds/          # Audio files (optional)
├── migrations/
│   └── 001_init.sql     # Database schema
├── .env                  # Database credentials (gitignored)
├── .env.example          # Template for credentials
└── Cargo.toml            # Dependencies
```

## Building for Distribution

### macOS
```bash
cargo build --release
# Binary at: target/release/two-people
```

### Windows (Cross-compile from macOS)
```bash
# Install Windows target
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

## Customization

### Adding Questions

Edit the `Questions` default implementation in `main.rs`:

```rust
Q { 
    title: "Your question...", 
    left: "OPTION\nA", 
    left_em: "emoji", 
    right: "OPTION\nB", 
    right_em: "emoji",
    trait_name: "trait_name" 
},
```

### Changing Colors

Modify the color constants at the top of `main.rs`:
- `BG_COLOR` - Background
- `CARD_LEFT` - Left card color
- `CARD_RIGHT` - Right card color
- etc.

## License

MIT License - Feel free to use this for your own "Which type are you?" games!

## Credits

- Inspired by "Cut" YouTube card games
- Inspired by viral TikTok "2 types of people" videos
- Built with [Bevy Engine](https://bevyengine.org/)

---

**Made with Rust and vibes**
