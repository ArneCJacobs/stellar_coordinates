
from astroquery.gaia import Gaia

def main():
    amount = 10
    amount = 2_000_000
    # B.r_med_photogeo as d, B.*
    # query = f"""
    # SELECT top {amount}
    # B.r_med_photogeo as d
    # FROM gaiaedr3.gaia_source AS G
    # JOIN external.gaiaedr3_distance AS B USING (source_id)
    # WHERE B.r_med_photogeo > 0
    # ORDER BY d
    # """

    # query = f"""
    # SELECT COUNT(*)
    # FROM gaiaedr3.gaia_source AS G
    # JOIN external.gaiaedr3_distance AS B USING (source_id)
    # """

    # query = f"""
    # SELECT top {amount}
    # edr3.l, edr3.b, r_med_geo , r_med_photogeo, r_med_geo as d
    # FROM gaiaedr3.gaia_source AS edr3
    # JOIN (
        # SELECT r_med_geo, r_med_photogeo
        # external.gaiaedr3_distance

    # ) using(source_id)
    # WHERE r_med_geo >= 0
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
    print(results)


if __name__ == '__main__':
    main()
