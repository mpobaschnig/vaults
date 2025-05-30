# Vaults release steps

This list is a reminder for things we need to take care of when releasing a new version of Vaults.

## Smoke test / QA

Some tests that we should do to see if something obvious stopped working:

CryFS:

* Add new vault
* Open vault
* Close vault

gocryptfs:

* Add new vault
* Open vault
* Close vault

General:

* Close Vaults app and start again to see if created vaults are loaded correctly
* Search for a vault
* Deleted one (gocryptfs or CryFS) vault
* Close Vaults app and start again to see if deleted vault was removed correctly
* Open and close the remaining vault again
* Delete last vault
* Close app and restart Vaults app again to see if main screen appears

##  Update translations

We use po files, so update it accordingly by either:

1. Call xgettext directly: `xgettext --files-from=po/POTFILES.in --from-code=UTF-8 --output po/vaults.pot`, or
2. Use `meson compile vaults-pot` within the meson build directory

## Update screenshots

In case of some UI changes, we should update the screenshots displaying the current state-of-the-art.

## Update metainfo description

The metainfo file contains information displayed on Flathub and sources that use it (e.g. GNOME Software).

## Prepare GitHub Release

After all these things, we can prepare the relase:
* Create release branch in the following format: `vaults-0.Y`
* Create tag on main in the following format: `0.Y.Z`
* Create tar by: `meson _build && ninja -C _build dist`
* Push branch + tag to Github
* Add tar to release

## Prepare Flathub Release

When we published our new release on GitHub, we can update our Flatpak. We need to update the Flatpak repository by adding new link, eventually updating dependencies, and SHA256 value.

## Post-release chore

When the new version was published on Flathub, we can finally update our branch to the next development version, i.e. bumping the minor version up by one within `meson.build` in the root directory and within the `cargo.toml` (0.Y+1.0).
