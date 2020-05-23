# Copyright 1999-2020 Gentoo Foundation
# Distributed under the terms of the GNU General Public License v2
# $Header: $

EAPI=7

CRATES="
{{ crates }}
"

inherit cargo
inherit check-reqs
inherit desktop

MY_PV="v${PV}"
MY_P="${PN}-${MY_PV}"

DESCRIPTION="A Rust application that displays a QR code from text"
HOMEPAGE="https://gitlab.com/claudiomattera/rust-qr-encoder/"
SRC_URI="
    https://gitlab.com/claudiomattera/${PN}/-/archive/${MY_PV}/${MY_P}.tar.bz2
    $(cargo_crate_uris ${CRATES})
"
RESTRICT="mirror"

LICENSE="MIT"
SLOT="0"
KEYWORDS="x86 amd64"

DEPEND="x11-libs/gtk+:3"
RDEPEND="x11-libs/gtk+:3"
BDEPEND="x11-libs/gtk+:3"

S="${WORKDIR}/${MY_P}"

CHECKREQS_DISK_BUILD="600M"

src_compile() {
    debug-print-function ${FUNCNAME} "$@"

    export CARGO_HOME="${ECARGO_HOME}"

    cargo build -v -j $(makeopts_jobs) $(usex debug "" --release) \
        || die "cargo build failed"
}

src_install() {
    debug-print-function ${FUNCNAME} "$@"

    cargo install --path . -j $(makeopts_jobs) --root="${D}/usr" $(usex debug --debug "") "$@" \
        || die "cargo install failed"
    rm -f "${D}/usr/.crates.toml"

    [ -d "${S}/man" ] && doman "${S}/man"

    newicon --size 32 ${FILESDIR}/icon-32.png rust-qr-encoder.png
    newicon --size 64 ${FILESDIR}/icon-64.png rust-qr-encoder.png
    newicon --size 128 ${FILESDIR}/icon-128.png rust-qr-encoder.png

    make_desktop_entry /usr/bin/rust-qr-encoder "QR encoder" rust-qr-encoder Graphics
}
