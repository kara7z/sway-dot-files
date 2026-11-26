# Dell Precision 5490 webcam on Void Linux

This optional configuration is for the OV01A10 sensor behind Intel IPU6/IVSC.
It was tested on Void x86_64 with kernel `7.3.6_1` and libcamera 0.7.1. The
regular dotfile installer does not install these hardware-specific settings.

Firefox uses **Built-in Front Camera** through PipeWire. V4L2 applications like
Discord use **Laptop Webcam** (`/dev/video48`), which requests the full
1280×800 sensor area and delivers YUYV at 30 fps. The relay starts the real
camera when an application starts capture and stops it when capture ends.

## Prerequisites

Install `gstreamer1-pipewire` and ensure the GStreamer `capssetter`,
`videoconvert`, `videorate`, and `v4l2sink` elements are available. The session
needs PipeWire's libcamera SPA plugin, working Intel camera firmware, and the
`v4l2loopback` module for the running kernel. Before setting up the relay,
`cam -l` should list the internal front camera and `wpctl status` should list a
libcamera source.

The tested Void kernel package omitted `mei-vsc-hw`, `mei-vsc`, `ivsc-csi`, and
`ivsc-ace`. If `modinfo` cannot find them, build them against the headers for
the running kernel. For `7.3.6_1`, install `linux7.3-headers`, extract Linux
7.3.6 from kernel.org, and run the following as your user from that source tree:

```bash
build=$(mktemp -d)
cp drivers/misc/mei/{*.h,vsc-tp.c,vsc-fw-loader.c,platform-vsc.c} "$build/"
cp drivers/media/pci/intel/ivsc/{mei_csi.c,mei_ace.c} "$build/"
{
    sed -n '/^obj-\$(CONFIG_INTEL_MEI_VSC_HW)/,$p' drivers/misc/mei/Makefile
    cat drivers/media/pci/intel/ivsc/Makefile
} > "$build/Kbuild"
kernel=$(uname -r)
make -C "/lib/modules/$kernel/build" M="$build" \
    CONFIG_INTEL_VSC=m CONFIG_INTEL_MEI_VSC=m CONFIG_INTEL_MEI_VSC_HW=m modules
sudo install -d "/lib/modules/$kernel/updates/precision-5490-camera"
sudo install -m 0644 "$build"/*.ko "/lib/modules/$kernel/updates/precision-5490-camera/"
sudo depmod -a "$kernel"
```

Repeat the build with matching source and headers if a later kernel still
omits these modules. Do not reuse the old `.ko` files.

## Install the camera configuration

If upgrading from the old `/etc/sv/webcam` service, stop it and move its runit
enablement link out of the active service directory. Back up any old
`89-ipu6-hide-video.rules` rule and loopback options before disabling them:
libcamera needs to be able to open the real IPU6 devices. Discord should use
its normal launcher without a `pw-v4l2` wrapper.

Install `v4l2-relayd` 0.2.0 with Ubuntu's libcamerasrc/SoftISP patch (Ubuntu
package `0.2.0-0ubuntu1.1` was tested). On Void x86_64 glibc the binary from
the release package works with the installed GStreamer libraries. Run from
the repo root:

```bash
files=hardware/precision-5490-webcam
(
    set -euo pipefail
    cd "$(mktemp -d)"
    package=v4l2-relayd_0.2.0-0ubuntu1.1_amd64.deb
    curl -fSLO "https://archive.ubuntu.com/ubuntu/pool/universe/v/v4l2-relayd/$package"
    printf '%s  %s\n' 8ec8bc21200f314f22f69314b5e05364f1fecce14588dfcb11dc659a0a198394 "$package" |
        sha256sum -c -
    ar p "$package" data.tar.zst | tar --zstd -xf - ./usr/bin/v4l2-relayd
    sudo install -D -m 0755 usr/bin/v4l2-relayd /usr/local/libexec/v4l2-relayd
)
sudo install -D -m 0755 "$files/start-laptop-webcam" /usr/local/libexec/start-laptop-webcam
sudo install -D -m 0644 "$files/70-laptop-webcam.conf" /etc/pipewire/pipewire.conf.d/70-laptop-webcam.conf
sudo install -D -m 0644 "$files/61-laptop-webcam.conf" /etc/wireplumber/wireplumber.conf.d/61-laptop-webcam.conf
sudo install -D -m 0644 "$files/modprobe.conf" /etc/modprobe.d/laptop-webcam.conf
sudo install -D -m 0644 "$files/modules-load.conf" /etc/modules-load.d/laptop-webcam.conf
sudo install -D -m 0644 "$files/90-precision-5490-camera.rules" /etc/udev/rules.d/90-precision-5490-camera.rules
```

Merge `configuration.yaml` into the libcamera configuration. For a
`/usr/local` libcamera build, the system path is
`/usr/local/etc/libcamera/configuration.yaml`; a per-user
`~/.config/libcamera/configuration.yaml` takes precedence. In Firefox,
set `media.webrtc.camera.allow-pipewire` to `true` in `about:config`.

For a fresh installation, reboot to apply the driver/module and udev
configuration together. Then check the compatibility camera:

```sh
v4l2-ctl -d /dev/video48 --get-fmt-video --get-parm
timeout 12 v4l2-ctl -d /dev/video48 --stream-mmap=3 --stream-count=40 --stream-to=/dev/null
```
