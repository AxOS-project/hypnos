pkgname='hypnos'
pkgver='0.1.0'
pkgrel='2'
pkgdesc='A Wayland idle time based notification and action daemon'
arch=('x86_64')
url='https://github.com/axos-project/hypnos'
license=('GPL')
depends=('rust' 'cargo' 'wayland-protocols' 'libnotify' 'systemd')
makedepends=('cargo')

build() {
    cd "$srcdir"
    cargo build --release
}

package() {
    install -Dm755 $srcdir/target/release/hypnos "$pkgdir/usr/bin/hypnos"
}
