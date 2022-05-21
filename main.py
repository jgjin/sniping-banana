import json

from sniping_banana import SnipingBanana


def get_config():
    with open("config.json") as config_json:
        return json.load(config_json)

    raise FileNotFoundError("Could not find config.json")


def main():
    config = get_config()

    sniping_banana = SnipingBanana(config)

    sniping_banana.wait()

    sniped = sniping_banana.shoot()
    if not sniped:
        print("Missed all compatible reservations!")


if __name__ == "__main__":
    main()
