from astroquery.gaia import Gaia
import matplotlib.pyplot as plt
import seaborn as sns
import pandas as ps
from math import pi
import numpy as np
import plotly.express as px
import os

DESTINATION_FILE = './stars_big.csv'
DESTINATION_FILE_COMPRESSED = f"stars_big_transformed.csv"

def download_data():
    # see columns in https://gea.esac.esa.int/archive/documentation/GDR2/Gaia_archive/chap_datamodel/sec_dm_main_tables/ssec_dm_gaia_source.html
    # query adapted from https://arxiv.org/abs/1905.13189v2 (https://doi.org/10.21105/astro.1905.13189)
    job = Gaia.launch_job("""
    select top 1000000
    l,b, radius_val
    FROM gaiadr2.gaia_source
    WHERE radius_val > 0
    ORDER BY RANDOM_INDEX
    """, dump_to_file=True, output_format='csv')

    file = job.outputFile
    os.rename(file, DESTINATION_FILE)


def to_cartesian_coordinates(data):

    r = data['radius_val']
    rho = data['l'] * 2 * pi / 360
    theta = (data['b'] + 90 ) * 2 * pi / 360

    # https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates
    data['x'] = r * np.cos(rho) * np.sin(theta)
    data['y'] = r * np.sin(rho) * np.sin(theta)
    data['z'] = r * np.cos(theta)
    return data


def download_and_transform():
    download_data()

    file = DESTINATION_FILE
    data = ps.read_csv(file)
    data = to_cartesian_coordinates(data)
    data.to_csv(DESTINATION_FILE_COMPRESSED)
    os.remove(DESTINATION_FILE)

    return data


def plot_lat_lon(resultset):
    # print(list(resultset['l']))
    x = (resultset['l']+ 180) % 360
    y = resultset['b']
    plt.clf()
    sns.histplot(
        x=x,
        y=y,
        palette=plt.cm.jet,
        bins=(200, 200)
    )

    plt.show()

def plot_3d_scatter(data):
    fig = px.scatter_3d(data, x='x', y='y', z='z')
    fig.show()

if __name__ == '__main__':

    if not os.path.exists(DESTINATION_FILE_COMPRESSED):
        data = download_and_transform()
    else:
        data = ps.read_csv(DESTINATION_FILE_COMPRESSED)

    # plot_3d_scatter(data)
    plot_lat_lon(data)

