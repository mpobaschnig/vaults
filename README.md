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

# How to build

The supported way of building Vaults using GNOME Builder (or Visual Studio Code with the Flatpak extension) by cloning the repository and simply building and then running it.

# Contributing

Pull requests for bugs or translation updates are welcomed!
However, before working on new features, please open a new topic in [discussions](https://github.com/mpobaschnig/vaults/discussions) or comment on a dedicated [issue](https://github.com/mpobaschnig/vaults/issues), so duplicate efforts can be prevented.
To coordinate accordingly, opening a pull request early as a draft is encouraged to make changes transparent to others.

# Bug reporting

For bug reports, please try to reproduce the issue with additional debug information enabled by running Vaults in the terminal using:

```bash
flatpak run --env=RUST_LOG=trace io.github.mpobaschnig.Vaults
```

When sharing, please be careful of redacting personally identifiable information (PII).

# Translations

Imagery is translated mainly using [Transifex](https://www.transifex.com/mpobaschnig/vaults/). Translations over Github pull requests are also accepted, if you prefer to do so.
