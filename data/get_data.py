from astroquery.gaia import Gaia
import matplotlib.pyplot as plt
import seaborn as sns
import pandas as ps
from math import pi
import numpy as np
import plotly.express as px
import os

DESTINATION_FILE = './stars_big.csv'
DESTINATION_FILE_COMPRESSED = f"stars_transformed.csv.gz"

def download_data():
    # see columns in https://gea.esac.esa.int/archive/documentation/GDR2/Gaia_archive/chap_datamodel/sec_dm_main_tables/ssec_dm_gaia_source.html
    # query adapted from https://arxiv.org/abs/1905.13189v2 (https://doi.org/10.21105/astro.1905.13189)
    # select top 1000000
    # FROM gaiadr2.gaia_source

    amount = 2_000_000
    # query = f"""
    # SELECT top {amount}
    # B.r_med_photogeo as d, G.l, G.b, B.*
    # FROM gaiaedr3.gaia_source AS G
    # JOIN external.gaiaedr3_distance AS B USING (source_id)
    # WHERE B.r_med_photogeo > 0
    # ORDER BY d
    # """

    query = f"""
    SELECT TOP {amount} l, b, e3d.r_med_geo as d
    FROM (
        SELECT  source_id, r_med_geo
        FROM external.gaiaedr3_distance
        WHERE r_med_geo > 0
        ORDER BY r_med_geo
    ) AS e3d
    JOIN gaiaedr3.gaia_source using(source_id)
    """

    results = Gaia.launch_job(query).get_results().to_pandas()

    # job = Gaia.launch_job("""
    # SELECT top 2000000
    # l,b, parallax, parallax_error
    # FROM gaiaedr3.gaia_source
    # WHERE
        # parallax IS NOT NULL AND
        # parallax_error IS NOT NULL AND
        # parallax_error < 20
    # ORDER BY RANDOM_INDEX
    # """, dump_to_file=True, output_format='csv')

    # file = job.outputFile
    # os.rename(file, DESTINATION_FILE)
    return results


def to_cartesian_coordinates(data):

    r = data['d']
    rho = data['l'] * 2 * pi / 360
    theta = (data['b'] + 90 ) * 2 * pi / 360

    # https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates
    data['x'] = r * np.cos(rho) * np.sin(theta)
    data['y'] = r * np.sin(rho) * np.sin(theta)
    data['z'] = r * np.cos(theta)
    return data

def download_and_transform():
    print("downloading data")
    data = download_data()

    # file = DESTINATION_FILE
    # print("reading data")
    # data = ps.read_csv(file)
    print(len(data))
    print("converting to Cartesian coordinates")
    data = to_cartesian_coordinates(data)
    # data.to_json(DESTINATION_FILE_COMPRESSED, orient='records')
    print("writing to compressed file")
    data.to_csv(DESTINATION_FILE_COMPRESSED)
    # os.remove(DESTINATION_FILE)

    return data


def plot_lat_lon(resultset):
    # print(list(resultset['l']))
    x = (resultset['l']+ 180) % 360
    y = resultset['b']
    plt.figure()
    plt.clf()
    sns.histplot(
        x=x,
        y=y,
        cmap=plt.cm.jet,
        bins=(200, 200)
    )

    plt.show(block=False)

def plot_3d_scatter(data):
    fig = px.scatter_3d(data, x='x', y='y', z='z')
    fig.show()

def plot_distance_hist(data):
    # dist = np.sqrt( data['x'] ** 2 + data['y'] ** 2 + data['z'] ** 2)
    # # fig = px.histogram(x=dist)
    # # fig.update_xaxes(range=[0, np.max(dist)])
    # # fig.show()
    bounds = [
        np.percentile(data, 2),
        100,
    ]
    plt.figure()
    sns.histplot(x=data['d'], bins=1000, binrange=bounds)
    plt.show(block=False)
    # print([0, np.percentile(data['d'], 95)])
    # fig = px.histogram(data, x='d', nbins=100)
    # fig.update_xaxes(range=[0, 10])
    # fig.show()

    # plt.figure()
    # sns.histplot(x=data['d'])
    # plt.show(block=False)

def plot_parallax_error(data):
    plt.figure()
    plt.clf()
    sns.histplot(
        data=data,
        x='parallax',
        y='parallax_error',
        cmap=plt.cm.jet
    )
    plt.show(block=False)


if __name__ == '__main__':
    # data = ps.read_csv("./stars.csv.gz")

    if not os.path.exists(DESTINATION_FILE_COMPRESSED):
        data = download_and_transform()
    else:
        data = ps.read_csv(DESTINATION_FILE_COMPRESSED)

    # plot_3d_scatter(data)
    plot_lat_lon(data)
    # plot_distance_hist(data)
    # plot_parallax_error(data)
    plt.show()




