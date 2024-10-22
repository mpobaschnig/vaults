global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('PKGDATADIR', pkgdatadir)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('VERSION', version + version_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR', localedir)
config = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)
# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.build_root() / 'src' / 'config.rs',
  meson.source_root() / 'src' / 'config.rs',
  check: true
)

sources = files(
  'backend/cryfs.rs',
  'backend/gocryptfs.rs',
  'backend/mod.rs',

  'ui/pages/mod.rs',
  'ui/pages/vaults_page_row.rs',
  'ui/pages/vaults_page_row_settings_window.rs',
  'ui/pages/vaults_page_row_password_prompt_window.rs',

  'ui/add_new_vault_window.rs',
  'ui/import_vault_window.rs',
  'ui/mod.rs',
  'ui/window.rs',

  'application.rs',
  'config.rs',
  'flatpak_info_parser.rs',
  'global_config_manager.rs',
  'main.rs',
  'vault.rs',
  'user_config_manager.rs',

  # Start of the new stuff
  'backend/cryfs/cryfs.rs',
  'backend/cryfs/errors.rs',
  'backend/gocryptfs/errors.rs',
  'backend/gocryptfs/gocryptfs.rs',
  'backend/encryption_backend.rs',
  'configuration/reader.rs',
  'configuration/writer.rs',
  'vault/settings.rs',
  'vault/vault.rs',
  'flatpak_info_parser.rs',
  'preferences.rs',
)

custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  depends: resources,
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    profile,
    meson.project_name(),
  ]
)
