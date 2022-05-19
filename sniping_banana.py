from datetime import datetime
import json
import time
from typing import List, Optional

import requests


class SnipingBanana:
    def __init__(self, config: dict):
        assert "auth" in config
        self.__init_headers(config["auth"])

        assert "reqs" in config
        self.__init_reqs(config["reqs"])

        self.payment_method = self.__get_default_payment_method()
        if self.payment_method is None:
            raise Exception("Could not get default payment method")

        self.wait_till = None
        if "wait_till" in config:
            self.wait_till = datetime.fromisoformat(config["wait_till"])

    def wait(self):
        now = datetime.now()
        if self.wait_till is not None and self.wait_till > now:
            secs = (self.wait_till - now).total_seconds()
            print(f"Sleeping {secs}s till {self.wait_till}")
            time.sleep(secs)

    def shoot(self) -> requests.Response:
        slots = self.__get_slots()
        slot = next(filter(lambda slot: self.reqs.satisfied_by(slot), slots),
                    None)
        if slot is None:
            raise Exception("Could not find satisfactory slot")

        book_token = self.__get_book_token(slot)

        return self.__book(book_token)

    def __init_headers(self, auth_config: dict):
        assert "api_key" in auth_config
        assert "auth_token" in auth_config

        self.headers = {
            "Authorization": f"ResyAPI api_key=\"{auth_config['api_key']}\"",
            "X-Resy-Auth-Token": auth_config["auth_token"],
        }

    def __init_reqs(self, reqs_config: dict):
        assert "venue_id" in reqs_config
        assert "date" in reqs_config
        assert "earliest_time" in reqs_config
        assert "party_size" in reqs_config

        self.reqs = ReservationRequirements(
            reqs_config["venue_id"],
            reqs_config["date"],
            reqs_config["earliest_time"],
            reqs_config["party_size"],
        )

    def __get_default_payment_method(self) -> Optional[int]:
        url = "https://api.resy.com/2/user"
        response = requests.get(url, headers=self.headers).json()

        if "payment_methods" not in response:
            return None

        payment_methods = response["payment_methods"]
        default_payment_method = next(
            filter(lambda payment_method: payment_method["is_default"],
                   payment_methods), None)
        if default_payment_method is not None:
            return default_payment_method["id"]

        return None

    def __get_slots(self) -> List[dict]:
        url = "https://api.resy.com/4/find"
        params = {
            "lat": 0,
            "long": 0,
            "venue_id": self.reqs.venue_id,
            "day": self.reqs.date,
            "party_size": self.reqs.party_size,
        }
        response = requests.get(url, params=params,
                                headers=self.headers).json()

        return response["results"]["venues"][0]["slots"]

    def __get_book_token(self, slot: dict) -> str:
        url = "https://api.resy.com/3/details"
        data = {
            "config_id": slot["config"]["token"],
            "day": self.reqs.date,
            "party_size": self.reqs.party_size,
        }
        response = requests.post(url, headers=self.headers, json=data).json()

        return response["book_token"]["value"]

    def __book(self, book_token: str) -> requests.Response:
        url = "https://api.resy.com/3/book"
        data = {
            "book_token": book_token,
            "struct_payment_method": json.dumps({
                "id": self.payment_method,
            }),
        }

        return requests.post(url, headers=self.headers, data=data)


class ReservationRequirements:
    def __init__(self, venue_id: int, date: str, earliest_time: str,
                 party_size: int):
        self.venue_id = venue_id
        self.date = date
        self.earliest_time = datetime.fromisoformat(
            f"{self.date} {earliest_time}")
        self.party_size = party_size

    def satisfied_by(self, slot: dict) -> bool:
        max_size = slot["size"]["max"]
        start = datetime.fromisoformat(slot["date"]["start"])

        return max_size > self.party_size and start >= self.earliest_time