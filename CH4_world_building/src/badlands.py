import bpy
import random
import math
import vnoise

# Initialize the vnoise Perlin noise generator
noise_gen = vnoise.Noise()
noise_gen.frequency = 0.2  # Control the frequency of the noise
noise_gen.octaves = 4      # Number of noise octaves for detail
noise_gen.lacunarity = 2.0 # Control the detail scaling
noise_gen.persistence = 0.5 # Influence of lower octaves

# Function to generate badlands-like rock formations
def create_rock_formation(location, scale, detail=10, height_multiplier=10):
    # Create a grid mesh
    bpy.ops.mesh.primitive_grid_add(size=scale, x_subdivisions=detail * scale, y_subdivisions=detail * scale, location=location)
    grid = bpy.context.object

    # Apply Perlin noise to the vertices
    mesh = grid.data
    for vertex in mesh.vertices:
        # Generate noise and exaggerate steepness
        noise_value = noise_gen.noise2(vertex.co.x, vertex.co.y)
        cliff_noise = abs(noise_value) ** 2.0  # Steeper cliffs
        vertex.co.z += cliff_noise * height_multiplier

    # Add a displacement modifier for additional randomness
    bpy.ops.object.modifier_add(type='DISPLACE')
    displace = grid.modifiers[-1]
    displace.strength = random.uniform(0.5, 1.5)

    # Apply smooth shading for a natural look
    bpy.ops.object.shade_smooth()

    return grid

# Function to create a shipping container
def create_shipping_container(location, size, rotation):
    # Add a cube
    bpy.ops.mesh.primitive_cube_add(size=1, location=location)
    container = bpy.context.object

    # Scale the cube to match container proportions
    container.scale.x = size[0]
    container.scale.y = size[1]
    container.scale.z = size[2]

    # Apply random rotation
    container.rotation_euler = (
        math.radians(rotation[0]),
        math.radians(rotation[1]),
        math.radians(rotation[2]),
    )

    # Assign a random material
    material = bpy.data.materials.new(name="ContainerMaterial")
    material.diffuse_color = (random.random(), random.random(), random.random(), 1)
    container.data.materials.append(material)

    return container

# Main function
def generate_scene():
    # Clear existing objects
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete(use_global=False)

    # Create the rock formations
    for _ in range(5):  # Generate 5 rock formations
        location = (random.uniform(-20, 20), random.uniform(-20, 20), 0)
        scale = random.uniform(5, 15)
        height_multiplier = random.uniform(15, 25)  # Increase height for more drama
        create_rock_formation(location, scale, detail=15, height_multiplier=height_multiplier)

    # Create the shipping containers
    for _ in range(10):  # Generate 10 shipping containers
        location = (random.uniform(-20, 20), random.uniform(-20, 20), random.uniform(0, 5))
        size = (random.uniform(2, 4), random.uniform(2, 4), random.uniform(2, 4))
        rotation = (0, 0, random.uniform(0, 360))  # Rotate only on the Z-axis
        create_shipping_container(location, size, rotation)

# Run the script
generate_scene()
