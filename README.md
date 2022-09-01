# stellar_coordinates

A 3D visualization of stars from [Gaia EDR 3](https://gea.esac.esa.int/archive/) written in Rust using [Bevy](https://bevyengine.org/).

Can handle the visualization of up to millions of stars.

Examples:
![Image](./figures/DeepinScreenshot_stellar_coordinates_test_20220805113909.png)
![Image](./figures/DeepinScreenshot_stellar_coordinates_test_20220805114003.png)

## Getting data

Go to <https://gaia.ari.uni-heidelberg.de/gaiasky/files/repository/catalog/dr3/> and download a `*.tar.gz` file and then extract the file afterwards.

For exmaple:
```bash
wget https://gaia.ari.uni-heidelberg.de/gaiasky/files/repository/catalog/dr3/001-small/v01_20220623/catalog-gaia-dr3-small.tar.gz
tar -xf catalog-gaia-dr3-small.tar.gz --one-top-level 
```

## Running the code

This project is build using `rust` and to run it you will need a working [Cargo](https://doc.rust-lang.org/cargo/)
installation. See <https://forge.rust-lang.org/infra/other-installation-methods.html>
for installation instructions.

If you have all the requirements you can run this project with

```bash
cargo run --release PATH_TO_CATALOG_DIRECTORY
```

### Fast compilation

See [Bevy setup](https://bevyengine.org/learn/book/getting-started/setup/) for
more information.

## Controls

Use W/A/S/D for strafing forward, left, back and right respectively.
Press the Space and Shift key to move up and down
Use the mouse to look around, click in the window to lock in mouse, press ESCAPE
to release the mouse.
