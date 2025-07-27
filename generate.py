import json
import random

def generate_galaxy_data():
    galaxy_names = [
        "Andromeda", "Milky Way", "Triangulum", "Whirlpool", "Sombrero",
        "Pinwheel", "Cartwheel", "Tuscana", "Centaurus", "Virgo",
        "Hercules", "Draco", "Phoenix", "Lynx", "Sculptor",
        "Fornax", "Aquarius", "Pegasus", "Ursa Major", "Ursa Minor",
        "Cassiopeia", "Perseus", "Orion", "Cygnus", "Leo",
        "Taurus", "Scorpius", "Sagittarius", "Capricornus", "Aquila",
        "Canis Major", "Canis Minor", "Bootes", "Lyra", "Crux",
        "Pavo", "Phoenix", "Hydra", "Columba", "Lupus",
        "Centaurus", "Ara", "Vulpecula", "Serpens", "Aquila"
    ]

    planet_names = [
        "Zyphor", "Krypton", "Blargon", "Xandar", "Vulcan",
        "Naboo", "Hoth", "Tatooine", "Dagobah", "Endor",
        "Mustafar", "Jakku", "Kashyyyk", "Coruscant", "Bespin",
        "Dantooine", "Geonosis", "Ryloth", "Mon Cala", "Lothal",
        "Felucia", "Zonama Sekot", "Alderaan", "Bespin", "Jakku",
        "Kessel", "Mandalore", "Raxus Prime", "Sullust", "Tatooine",
        "Yavin", "Dathomir", "Korriban", "Balmorra", "Zakuul",
        "Zakuul", "Korriban", "Taris", "Naboo", "Hoth",
        "Rishi", "Lothal", "Bespin", "Dantooine", "Geonosis",
        "Ryloth", "Mon Cala", "Felucia", "Alderaan", "Kashyyyk"
    ]
    planet_types = ["Gas", "Terrestrial", "Ocean"]
    galaxies = []

    for _ in range(90):  # Generate between 1 and 5 galaxies
        galaxy_name = random.choice(galaxy_names)
        pos = [round(random.uniform(0, 30), 2) for _ in range(2)]
        planets = []

        for _ in range(random.choices([1, 2, 3, 4, 5, 6], weights=[0.2, 0.2, 0.2, 0.4, 0.1, 0.1])[0]):  # Generate between 2 and 6 planets
            planet_name = random.choice(planet_names)
            x = round(random.uniform(0, 100), 2)
            y = round(random.uniform(0, 100), 2)
            radius = round(random.uniform(1, 10), 2)
            has_event = random.random() < (1 / 10)  # 1 in 8 probability
            has_component = random.random() < (1 / 8)  # 1 in 10 probability
            crystals = random.choices([0, 1, 2, 3], weights=[0.6, 0.2, 0.15, 0.05])[0]  # 0 most probable
            fuel = random.choices([0, 1], weights=[0.95, 0.05])[0]  # 0 most probable
            planet_type = random.choice(planet_types)  # Randomly select planet type

            planet = {
                "name": planet_name,
                "x": x,
                "y": y,
                "radius": radius,
                "planet_type": planet_type,
                "has_event": has_event,
                "has_component": has_component,
                "crystals": crystals,
                "fuel": fuel,
                "visited_by": [],
            }
            planets.append(planet)

        galaxy = {
            "name": galaxy_name,
            "pos": pos,
            "planets": planets
        }
        galaxies.append(galaxy)

    return galaxies

# Generate the galaxy data
galaxy_data = generate_galaxy_data()

# Convert to JSON and print
print(json.dumps(galaxy_data, indent=4))

