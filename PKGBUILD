pkgname='hypnos'
pkgver='1.0.0'
pkgrel='2'
pkgdesc='A Wayland idle time based action daemon'
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

YELLOW='\e[1;33m'
NC='\e[0m'

post_upgrade() {
    echo -e "${YELLOW}>>> IMPORTANT: We recommend updating the Hypnos service file.${NC}"
    echo -e "${YELLOW}>>> To update your local user service file, please run: hypnos install${NC}"
}