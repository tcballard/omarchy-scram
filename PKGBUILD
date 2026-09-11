# Run makepkg -si from the extracted source directory as an ordinary user.
pkgname=omarchy-scram
pkgver=0.2.1
pkgrel=1
pkgdesc='An original native maze chase for Omarchy Arcade'
arch=('x86_64' 'aarch64')
license=('MIT')
provides=("omarchy-munch=$pkgver")
conflicts=('omarchy-munch')
replaces=('omarchy-munch')
depends=('gcc-libs' 'glibc' 'libglvnd' 'libx11' 'libxcursor' 'libxi'
         'libxrandr' 'libxkbcommon' 'libxkbcommon-x11' 'wayland' 'dbus')
optdepends=('libpulse: optional original sound cues via paplay')
makedepends=('rust')
source=()
sha256sums=()

build() {
  cd "$startdir"
  cargo build --release --locked
}

check() {
  cd "$startdir"
  cargo test --locked
}

package() {
  cd "$startdir"
  install -Dm755 target/release/omarchy-scram "$pkgdir/usr/bin/omarchy-scram"
  ln -s omarchy-scram "$pkgdir/usr/bin/omarchy-munch"
  install -Dm644 packaging/io.github.tcballard.omarchy-scram.desktop \
    "$pkgdir/usr/share/applications/io.github.tcballard.omarchy-scram.desktop"
  install -Dm644 assets/packs/latch/icon.png \
    "$pkgdir/usr/share/icons/hicolor/128x128/apps/io.github.tcballard.omarchy-scram.png"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 THIRD_PARTY_LICENSES.txt "$pkgdir/usr/share/licenses/$pkgname/THIRD_PARTY_LICENSES.txt"
  install -Dm644 docs/ARTWORK-PACKS.md "$pkgdir/usr/share/doc/$pkgname/ARTWORK-PACKS.md"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}
