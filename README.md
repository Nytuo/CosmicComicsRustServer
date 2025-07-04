<h1 align="center">
  <a href="https://github.com/Nytuo/CosmicComics">
    <img src="https://nytuo.fr/images/LogoStretch_cc.png" alt="Logo" width="auto" height="100">
  </a>
</h1>
<div align="center">
<h2>CosmicComics Rust Server</h2>
  This project is part of <a href="https://github.com/Nytuo/CosmicComics">CosmicComics</a>
  <br />
  <br />
  <a href="https://github.com/NytuoIndustries/CosmicComicsNodeServer/issues/new?assignees=&labels=bug&template=01_BUG_REPORT.md&title=bug%3A+">Report a Bug</a>
  ·
  <a href="https://github.com/NytuoIndustries/CosmicComicsNodeServer/issues/new?assignees=&labels=enhancement&template=02_FEATURE_REQUEST.md&title=feat%3A+">Request a Feature</a>
  .<a href="https://github.com/Nytuo/CosmicComics/discussions">Ask a Question</a>

</div>

<div align="center">
<br />

[![Project license](https://img.shields.io/github/license/Nytuo/CosmicComics.svg?style=flat-square)](LICENSE)

[![code with love by Nytuo](https://img.shields.io/badge/%3C%2F%3E%20with%20%E2%99%A5%20by-Nytuo-ff1414.svg?style=flat-square)](https://github.com/Nytuo)

[![cosmiccomics](https://snapcraft.io/cosmiccomics/badge.svg)](https://snapcraft.io/cosmiccomics)

[![cosmiccomics](https://snapcraft.io/cosmiccomics/trending.svg?name=0)](https://snapcraft.io/cosmiccomics)

</div>

<details open="open">
<summary>Table of Contents</summary>

- [About](#about)
  - [Related repositories](#related-repositories)
  - [General description](#general-description)
  - [Built With](#built-with)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Usage](#usage)
- [Roadmap](#roadmap)
- [Support](#support)
- [Contributing](#contributing)
- [Authors \& contributors](#authors--contributors)
- [Security](#security)
- [License](#license)
- [Acknowledgements](#acknowledgements)

</details>

---

## About

Cosmic Comics is a Comics and Mangas reader and collectionner.  

### Related repositories
Cosmic Comics is divided in three categories
- Server
- Interface
- Application

Accross multiple repository
- [CosmicComics](https://github.com/Nytuo/CosmicComics) (Application)
- [CosmicComics Rust Server](https://github.com/Nytuo/CosmicComicsRustServer) (Server)
- [CosmicComics React Client](https://github.com/NytuoIndustries/CosmicComicsReactClient) (Interface)

### General description
The information about the series and books are provided by some API or manually set.
This is all you can do with this software and more:
- Read `CBR`, `CBZ`, `CB7`, `CBT`, `ZIP`, `RAR`, `7z`, `TAR`, `PDF`, `EPUB`, `Folder` which contains `PNG`, `JPG`, `JPEG`, `BMP`.
- Display your books and navigate your folders with custom covers (automaticatly by extraction or manually set)
- Set your books as `Read`, `Unread`, `Reading` or `favorite` and `note` them.
- Many Customizable parameters
- Zoom, Auto Background Color, Double page Mode, Blank first page, No double page for Horizontal, Manga Mode, Webtoon Mode, fullscreen, rotations, Bookmarks, Slideshow, SideBar, Hide Menu Bar.
- Display information about Comics/ Manga
- Libraries information provided by APIs (Marvel API, Google Books API, Anilist,...)
- Continue reading where you stopped and more...

### Built With
<div style="display: flex; align-item: center">
<img src="https://img.shields.io/badge/Rust-black?style=for-the-badge&logo=rust"/>
</div>

## Getting Started

### Prerequisites

Have Rust installed. You may also need PDFium to have PDF support

### Installation

See the <a href="https://github.com/Nytuo/CosmicComics/wiki/Installation">Installation</a> section of the Wiki

## Usage

See the <a href="https://github.com/Nytuo/CosmicComics/wiki/How-to-use">How to use</a> section of the Wiki

## Roadmap

See the [open issues](https://github.com/Nytuo/CosmicComicsRustServer/issues) for a list of proposed features (and known issues).

- [Top Feature Requests](https://github.com/NytuoIndustries/CosmicComicsRustServer/issues?q=label%3Aenhancement+is%3Aopen+sort%3Areactions-%2B1-desc) (Add your votes using the 👍 reaction)
- [Top Bugs](https://github.com/NytuoIndustries/CosmicComicsRustServer/issues?q=is%3Aissue+is%3Aopen+label%3Abug+sort%3Areactions-%2B1-desc) (Add your votes using the 👍 reaction)
- [Newest Bugs](https://github.com/NytuoIndustries/CosmicComicsRustServer/issues?q=is%3Aopen+is%3Aissue+label%3Abug)

## Support

Reach out to the maintainer at one of the following places:

- [GitHub Discussions](https://github.com/Nytuo/CosmicComics/discussions)
- Contact options listed on [this GitHub profile](https://github.com/Nytuo)


## Contributing

First off, thanks for taking the time to contribute! Contributions are what make the open-source community such an amazing place to learn, inspire, and create. Any contributions you make will benefit everybody else and are **greatly appreciated**.


Please read [our contribution guidelines](docs/CONTRIBUTING.md), and thank you for being involved!

## Authors & contributors

The original setup of this repository is by [Arnaud BEUX](https://github.com/Nytuo).

For a full list of all authors and contributors, see [the contributors page](https://github.com/Nytuo/CosmicComicsRustServer/contributors).

## Security

CosmicComics follows good practices of security, but 100% security cannot be assured.
CosmicComics is provided **"as is"** without any **warranty**. Use at your own risk.

_For more information and to report security issues, please refer to our [security documentation](docs/SECURITY.md)._

## License

This project is licensed under the **GNU General Public License v3**.

See [LICENSE](LICENSE) for more information.

## Acknowledgements

- All the Rust libraries authors   
- Plex and Jellyfin like server based media library for the inspiration
