from dataclasses import InitVar, dataclass
from math import ceil
from typing import ClassVar, Union


@dataclass
class SpringConfig:
    tension: float = 170
    friction: float = 26
    mass: float = 1
    precision: float = None
    velocity: float = None
    clamp: bool = False
    bounce: float = None

    @classmethod
    def gentle(cls):
        return SpringConfig(tension=120, friction=14)

    @classmethod
    def wobbly(cls):
        return SpringConfig(tension=180, friction=12)

    @classmethod
    def stiff(cls):
        return SpringConfig(tension=210, friction=20)

    @classmethod
    def slow(cls):
        return SpringConfig(tension=280, friction=60)

    @classmethod
    def molasses(cls):
        return SpringConfig(tension=280, friction=120)


@dataclass
class AnimatedValue:
    cur_pos: float = None
    from_pos: float = 0
    to_pos: float = 0
    last_vel: float = None

    def __post_init__(self):
        self.cur_pos = self.cur_pos or self.from_pos


def step_spring(
    dt_secs: float, val: AnimatedValue, config: SpringConfig = SpringConfig()
):
    finished = False
    v0 = config.velocity or 0

    precision = config.precision or (
        0.005
        if val.from_pos == val.to_pos
        else min(1, abs(val.to_pos - val.from_pos) * 0.001)
    )
    velocity = val.last_vel or v0

    # The velocity at which movement is essentially none
    rest_velocity = precision / 10

    # Bouncing is opt-in (not to be confused with overshooting)
    bounce_factor = 0 if config.clamp else config.bounce
    can_bounce = bounce_factor != 0

    # When `True`, the value is increasing over time
    is_growing = v0 > 0 if val.from_pos == val.to_pos else val.from_pos < val.to_pos

    # When `True`, the velocity is considered moving
    is_moving: bool

    # When `True`, the velocity is being deflected or clamped
    is_bouncing = False

    step = 1  # 1ms
    num_steps = ceil(dt_secs / (step / 1000))
    for _ in range(num_steps):
        is_moving = abs(velocity) > rest_velocity

        if not is_moving:
            finished = abs(val.to_pos - val.cur_pos) <= precision
            if finished:
                break

        if can_bounce:
            is_bouncing = (
                val.cur_pos == val.to_pos or val.cur_pos > val.to_pos == is_growing
            )

            # Invert the velocity with a magnitude, or clamp it.
            if is_bouncing:
                velocity = -velocity * bounce_factor
                val.cur_pos = val.to_pos

        spring_force = -config.tension * 0.000001 * (val.cur_pos - val.to_pos)
        damping_force = -config.friction * 0.001 * velocity
        acceleration = (spring_force + damping_force) / config.mass  # pt/ms^2

        velocity = velocity + acceleration * step  # pt/ms
        val.cur_pos = val.cur_pos + velocity * step

    val.last_vel = velocity
    return finished


if __name__ == "__main__":
    val = AnimatedValue(from_pos=200, to_pos=640)
    config = SpringConfig.gentle()

    for frame in range(500):
        if step_spring(dt_secs=16 / 1000, val=val, config=config):
            print(f"finished on frame {frame}")
            break

        print(val.cur_pos)
