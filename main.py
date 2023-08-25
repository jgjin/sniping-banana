import json

from sniping_banana import SnipingBanana
from timing_banana import TimingBanana


import http.client as http_client


def get_config():
    with open("config.json") as config_json:
        return json.load(config_json)

    raise FileNotFoundError("Could not find config.json")


def main():
    config = get_config()

    sniping_banana = SnipingBanana(config)
    timing_banana = TimingBanana(config, work=sniping_banana.shoot)

    successful = timing_banana.wait_and_run()
    if successful:
        print("Success!")
    else:
        print("Failed all attempts!")


if __name__ == "__main__":
    http_client.HTTPConnection.debuglevel = 0
    main()
