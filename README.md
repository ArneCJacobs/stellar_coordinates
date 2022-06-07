# stellar_coordinate_test

A 3D visualization of stars from [Gaia EDR 3](https://gea.esac.esa.int/archive/)
with distances taken from [VizieR Online Data Catalog](https://ui.adsabs.harvard.edu/abs/2021yCat.1352....0B/abstract#:~:text=VizieR%20Online%20Data%20Catalog%3A%20Distances,%2C%202021)%20%2D%20NASA%2FADS)

Can handle the visualization of at least 3 million stars.

## Getting data

You can either get the data programmatically be executing the python script in `data/get_data.py`
or from a Gaia EDR3 TAP endpoint ([example](https://gaia.ari.uni-heidelberg.de/tap.html))
with the following query:

```adql
SELECT TOP AMOUNT l, b, e3d.r_med_geo as d
FROM (
    SELECT  source_id, r_med_geo
    FROM external.gaiaedr3_distance
    WHERE r_med_geo > 0
    ORDER BY r_med_geo
) AS e3d
JOIN gaiaedr3.gaia_source using(source_id)
```

with `AMOUNT` replaced with the maximum amount of stars you want. Though the
endpoint will probably enforce a maximum of 2 or 3 million rows.

To execute the script make sure your current working directory is <./data>. Then
install the requirements with:

```bash
python3 -m pip install -r requirements.txt
```

 The script itself can then be executed with:

```bash
python3 get_data.py
```

This will download the data if it isn't downloaded already, and will also show a
density map based the longitude and latitude. Other visualization are available
in the script.

## Running the code

This project is build using `rust` and to run it you will need a working [Cargo](https://doc.rust-lang.org/cargo/)
installation. See <https://forge.rust-lang.org/infra/other-installation-methods.html>
for installation instructions.

If you have all the requirements you can run this project with

```bash
cargo run --release
```

### Fast compilation

See [Bevy setup](https://bevyengine.org/learn/book/getting-started/setup/) for
more information.

For fast compilation, the `lld` linker is required on linux systems and the `zld`
linker on maxOS. `zld` can be installed with bew:

```bash
brew install michaeleisel/zld/zld
```

If you don't want fast compile times, you can comment your platform in `./.cargo/config.toml`.

## Controls

Use W/A/S/D for strafing forward, left, back and right respectively.
Press the Space and Shift key to move up and down
Use the mouse to look around, click in the window to lock in mouse, pres ESCAPE
to release the mouse.

