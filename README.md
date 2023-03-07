<h1 align="center">
  <img src="data/icons/io.github.mpobaschnig.Vaults.svg" alt="Vaults" width="128" height="128"/><br>
  Vaults
</h1>

<p align="center"><strong>Keep important files safe</strong></p>

<p align="center">
 <a href="https://flathub.org/apps/details/io.github.mpobaschnig.Vaults"><img width="200" alt="Download on Flathub" src="https://flathub.org/assets/badges/flathub-badge-en.png"/></a>
</p>

<p align="center">
  <img src="data/resources/screenshots/main.png" alt="Main Window"/>
</p>

Vaults lets you create encrypted vaults in which you can safely store files.
It currently uses [gocryptfs](https://github.com/rfjakob/gocryptfs) and [CryFS](https://github.com/cryfs/cryfs/) for encryption.  Please always keep a backup of your encrypted files.

This version does not bundle [gocryptfs](https://github.com/rfjakob/gocryptfs) and [CryFS](https://github.com/cryfs/cryfs/) yet, so you need to install them on the host.

# How to build

Open GNOME Builder (or Visual Studio Code with the Flatpak extension), clone the repository, build and run it.

# Translations

From version 0.5.0 on, Vaults is also on [Transifex](https://www.transifex.com/mpobaschnig/vaults/).
I'm using this for the first time, so if there's anything left to do on my side to fully use Transifex, please open an issue.
Translations via Github PRs will still be accepted, if you prefer to do so.
