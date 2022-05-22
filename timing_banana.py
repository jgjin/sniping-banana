from datetime import datetime, timedelta
import time
from typing import Callable


class TimingBanana:
    def __init__(self, config: dict, work: Callable[[], bool]):
        self.__init_retry(config)

        self.wait_till = None
        if "wait_till" in config:
            self.wait_till = datetime.fromisoformat(config["wait_till"])

        self.work = work

    def wait_and_run(self) -> bool:
        if self.wait_till is not None:
            self.__sleep_till(self.wait_till)

        next_time = datetime.now()
        for attempt in range(self.max_num_attempts):
            print(f"---Attempt {attempt}---")

            successful = self.work()
            if successful:
                return True

            if attempt < (self.max_num_attempts - 1):
                next_time += self.time_between_attempts
                self.__sleep_till(next_time)

        return False

    def __init_retry(self, config: dict):
        if "retry" in config:
            retry_config = config["retry"]
            assert "secs_between_attempts" in retry_config
            assert "max_num_attempts" in retry_config

            self.time_between_attempts = timedelta(
                seconds=retry_config["secs_between_attempts"])
            self.max_num_attempts = retry_config["max_num_attempts"]
        else:
            self.time_between_attempts = timedelta(seconds=60)
            self.max_num_attempts = 1

    def __sleep_till(self, wakeup_time: datetime):
        now = datetime.now()
        if wakeup_time > now:
            secs = (wakeup_time - now).total_seconds()
            print(f"Sleeping {secs}s till {wakeup_time}")
            time.sleep(secs)
