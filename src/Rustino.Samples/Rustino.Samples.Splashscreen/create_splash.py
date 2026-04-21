from PIL import Image, ImageDraw, ImageFont

# Create a 500x300 image with a gradient background
img = Image.new('RGB', (500, 300), color=(45, 45, 60))
draw = ImageDraw.Draw(img)

# Draw a gradient-like effect with rectangles
for i in range(300):
    color = (45 + i//4, 45 + i//4, 60 + i//3)
    draw.rectangle([(0, i), (500, i+1)], fill=color)

# Add centered text
try:
    font = ImageFont.truetype("arial.ttf", 48)
except:
    font = ImageFont.load_default()

text = "Rustino"
bbox = draw.textbbox((0, 0), text, font=font)
text_width = bbox[2] - bbox[0]
text_height = bbox[3] - bbox[1]
position = ((500 - text_width) // 2, (300 - text_height) // 2 - 20)
draw.text(position, text, fill=(255, 255, 255), font=font)

# Add smaller subtitle
try:
    small_font = ImageFont.truetype("arial.ttf", 20)
except:
    small_font = ImageFont.load_default()

subtitle = "Loading..."
bbox = draw.textbbox((0, 0), subtitle, font=small_font)
sub_width = bbox[2] - bbox[0]
sub_position = ((500 - sub_width) // 2, (300 - text_height) // 2 + 30)
draw.text(sub_position, subtitle, fill=(200, 200, 200), font=small_font)

# Save the image
img.save('splash.png')
print("Created splash.png")
