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

    shot = sniping_banana.shoot()
    if shot.ok:
        print("Sniped!")
    else:
        print("Missed!")
    print(shot.text)


if __name__ == "__main__":
    main()
