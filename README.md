# League Auto-Accept

A cross-platform desktop application that automatically accepts League of Legends matches, so you never miss a game again.


## ✨ Features

- 🎯 **Automatic Match Acceptance** - Instantly accepts ready checks when detected
- 🔗 **League Client Integration** - Connects directly to the League of Legends client API
- 🎮 **Manual Accept Button** - Accept matches manually when needed
- 📊 **Real-time Status** - Visual indicators for connection and monitoring status
- 📝 **Activity Logging** - Track all auto-accept activity with timestamps
- 🌙 **Dark Theme** - League-inspired dark UI design
- 🖥️ **Cross-Platform** - Works on Windows, macOS, and Linux

## 🚀 Quick Start

### Prerequisites

- League of Legends must be installed and running
- No additional setup required - the app connects automatically

### Installation

#### Windows
1. Download `League Auto-Accept_0.1.0_x64-setup.exe` from the [Releases](../../releases) page
2. Run the installer and follow the setup wizard
3. Launch "League Auto-Accept" from your Start Menu

#### macOS
1. Download `League Auto-Accept_0.1.0_universal.dmg` from the [Releases](../../releases) page
2. Open the DMG file and drag the app to your Applications folder
3. Launch "League Auto-Accept" from Launchpad or Applications

#### Linux
1. Download the appropriate `.AppImage` or `.deb` file from the [Releases](../../releases) page
2. Make the AppImage executable: `chmod +x League-Auto-Accept_0.1.0_amd64.AppImage`
3. Run the application

## 🎮 How to Use

1. **Start League of Legends** - Make sure the League client is running
2. **Launch League Auto-Accept** - Open the application
3. **Connect** - Click "Connect to League" to establish connection
4. **Enable Auto-Accept** - Click "Enable Auto-Accept" to start monitoring
5. **Queue for a game** - The app will automatically accept when a match is found!

### Manual Operation

- **Manual Accept**: Use the "Manual Accept" button to accept the current ready check
- **Toggle Monitoring**: Enable/disable auto-accept as needed
- **Monitor Status**: Check connection and monitoring status in real-time

## 🔧 Technical Details

### How It Works

League Auto-Accept connects to the League of Legends client through its built-in REST API:
- Reads the client's lockfile to get connection details
- Monitors the `/lol-matchmaking/v1/ready-check` endpoint
- Automatically sends acceptance when a ready check is detected
- Handles reconnection if the League client restarts

### Security & Privacy

- **Local Only**: All communication stays on your computer
- **No Data Collection**: No personal information is transmitted or stored
- **Open Source**: Full source code is available for review
- **League API**: Uses official Riot Games client API endpoints

## 🛠️ Development

### Built With

- **Frontend**: React + TypeScript + Vite
- **Backend**: Rust
- **Framework**: Tauri v2
- **Styling**: CSS with League of Legends theme

### Prerequisites for Development

- [Node.js](https://nodejs.org/) (v18 or later)
- [Rust](https://rustup.rs/) (latest stable)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Setup

```bash
# Clone the repository
git clone git@github.com:SanderJevnaker/league-auto-accept.git
cd league-auto-accept

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Building

```bash
# Build for your platform
npm run tauri build

# Build for Windows (from macOS/Linux)
npm run tauri build -- --target x86_64-pc-windows-gnu

# Build universal macOS binary
npm run tauri build -- --target universal-apple-darwin
```

## 📋 Requirements

### System Requirements

- **Windows**: Windows 10 version 1903 or later
- **macOS**: macOS 10.15 Catalina or later
- **Linux**: Modern distribution with WebKit2GTK

### League of Legends

- League of Legends must be installed and running
- Works with all regions and game modes
- Compatible with both old and new League clients

## ❓ Troubleshooting

### Connection Issues

**"League Client lockfile not found"**
- Ensure League of Legends is running
- Try restarting the League client
- Check if League is installed in a custom location

**"Failed to connect to League Client"**
- League client may be starting up - wait and try again
- Restart both League and the auto-accept app
- Check if another app is interfering with League's API

### Auto-Accept Not Working

**"Auto-accept enabled but matches not accepted"**
- Verify the connection status shows "Connected"
- Check the activity log for error messages
- Try using "Manual Accept" to test the connection


## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This application is not affiliated with Riot Games. League of Legends is a trademark of Riot Games, Inc.

**Use at your own risk**: While this tool uses official League client APIs and should be safe, any third-party tool carries inherent risks. The developers are not responsible for any account actions taken by Riot Games.

## 🙏 Acknowledgments

- [Riot Games](https://www.riotgames.com/) for providing client APIs
- [Tauri](https://tauri.app/) for the amazing cross-platform framework
- [League of Legends community](https://www.reddit.com/r/leagueoflegends/) for inspiration and feedback

## 📞 Support

- 🐛 **Bug Reports**: [Create an Issue](../../issues/new?template=bug_report.md)
- 💡 **Feature Requests**: [Create an Issue](../../issues/new?template=feature_request.md)
- 💬 **Questions**: [Discussions](../../discussions)

---

**Made with ❤️ for the League of Legends community**