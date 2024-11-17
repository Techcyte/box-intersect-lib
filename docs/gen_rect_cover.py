import heapq
import math
import os
from PIL import Image, ImageDraw, ImageFont
import PIL
import numpy as np

IMG_SZ = 400
regular = (0,0,0,int(255*0.2))
selected = (0,0,0,int(255))
TILE_SIZE = 200


def contains(outer, inner):
    ldiff = inner[0] >= outer[0]
    tdiff = inner[1] >= outer[1]
    wdiff = outer[2] >= inner[2]
    hdiff = outer[3] >= inner[3]
    return ldiff & tdiff & wdiff & hdiff


def generate_data(num_boxes, region_size, max_box_size):
    assert max_box_size > 0 and num_boxes > 0 and region_size > 0
    assert region_size > max_box_size, "Region size must be larger than the max box size"
    box_positions = np.random.randint(low=0, high=region_size-max_box_size, size=(num_boxes, 2), dtype="int32")
    box_sizes = np.random.randint(low=6, high=max_box_size, size=(num_boxes, 2), dtype="int32")
    boxes = np.concatenate([box_positions, box_positions+box_sizes], axis=1)
    return boxes


def generate_data_norm(num_boxes, region_size, max_box_size):
    assert max_box_size > 0 and num_boxes > 0 and region_size > 0
    assert region_size > max_box_size, "Region size must be larger than the max box size"
    box_positions = np.random.normal(loc=region_size/2, scale=region_size/6, size=(num_boxes, 2)).astype(dtype="int32")
    box_positions = np.maximum(0, np.minimum(box_positions, region_size))
    box_sizes = np.random.randint(low=6, high=max_box_size, size=(num_boxes, 2), dtype="int32")
    boxes = np.concatenate([box_positions, box_positions+box_sizes], axis=1)
    return boxes


def draw_cbox(imgdraw, box, color=selected, width=1, rotate=0):
    x1, y1, x2, y2 = box
    if rotate:
        x1, y1, x2, y2 = y1, x1, y2, x2
    imgdraw.rectangle(((x1, y1), (x2, y2)), outline=color, fill=None, width=width)


def generate_rect_cover(boxes):
    # fnt = ImageFont.FreeTypeFont(size=20)
    # draw text, half opacity
    base = Image.new('RGBA', (IMG_SZ, IMG_SZ), (255,255,255,255))
    draw = ImageDraw.Draw(base)

    for b in boxes:
        draw_cbox(draw, b, color='black',width=1,rotate=0)
    
    for i in range(5):
        yield base

    boxes = boxes[np.argsort(boxes[:,0])]
    # taken_boxes = np.zeros(len(boxes), dtype="bool")
    taken_tiles = []
    left_boxes = boxes

    while len(left_boxes):
        b1 = left_boxes[np.argmin(left_boxes[:,0])]
        # if taken_boxes[i1]:
        #     continue
        min_y = b1[1]
        max_y = b1[3]
        region = [
            b1[0],
            max_y - TILE_SIZE,
            b1[0] + TILE_SIZE,
            min_y + TILE_SIZE, 
        ]
        base = Image.new('RGBA', (IMG_SZ, IMG_SZ), (255,255,255,255))
        draw = ImageDraw.Draw(base)

        for b in left_boxes:
            draw_cbox(draw, b, color='black',width=1,rotate=0)
        
        for b in taken_tiles:
            draw_cbox(draw, b, color='green',width=1,rotate=0)

        draw.line([(b1[0], 0), (b1[0], IMG_SZ)], fill="gray", width=4)

        tile_overlap = Image.new('RGBA', (IMG_SZ, IMG_SZ), (255, 255, 255, 0))
        tile_overlap_draw = ImageDraw.Draw(tile_overlap)
        draw_cbox(tile_overlap_draw, region, color='blue',width=4,rotate=0)
        yield Image.alpha_composite(base, tile_overlap)

        region_contained_boxes = left_boxes[contains(region, left_boxes.T)]

        # sort by x coordinate
        region_contained_boxes = region_contained_boxes[np.argsort(region_contained_boxes[:,0])]        
        assert len(region_contained_boxes) > 0


        for b2 in region_contained_boxes:
            if contains(region, b2):
                min_y = min(min_y, b2[1])
                max_y = max(max_y, b2[3])
                region = [
                    b1[0],
                    max_y - TILE_SIZE,
                    b1[0] + TILE_SIZE,
                    min_y + TILE_SIZE, 
                ]
                tile_overlap = Image.new('RGBA', (IMG_SZ, IMG_SZ), (255, 255, 255, 0))
                tile_overlap_draw = ImageDraw.Draw(tile_overlap)

                draw_cbox(draw, b2, color='green',width=2,rotate=0)
                draw_cbox(tile_overlap_draw, region, color='blue',width=4,rotate=0)

                yield Image.alpha_composite(base, tile_overlap)
            
        gen_tile = [
            region[0],
            region[1],
            region[0]+TILE_SIZE,
            region[1]+TILE_SIZE,
        ]
        draw_cbox(draw, gen_tile, color='green',width=2,rotate=0)
        for i in range(3):
            yield base
        # taken_boxes |= contains(gen_tile, boxes.T)
        left_boxes = left_boxes[~contains(gen_tile, left_boxes.T)]
        assert contains(gen_tile, b1), f"{gen_tile}, {b1}"
        
        taken_tiles.append(gen_tile)

    base = Image.new('RGBA', (IMG_SZ, IMG_SZ), (255,255,255,255))
    draw = ImageDraw.Draw(base)

    for b in boxes:
        draw_cbox(draw, b, color='black',width=1,rotate=0)
    
    for b in taken_tiles:
        b[0] -= 1
        b[1] -= 1
        draw_cbox(draw, b, color='green',width=1,rotate=0)

    for i in range(50):
        yield base


def dup_images_for_slowness(img_gen):
    for img in img_gen:
        for i in range(20):
            yield img

def save_line_search_gif(fname, boxes):
    img_generator = dup_images_for_slowness(generate_rect_cover(boxes))
    img1 = next(img_generator)

    img1.save(f'docs/{fname}',
                save_all=True, append_images=img_generator, optimize=True, duration=6, loop=0)


IMG_SZ = 600
TILE_SIZE = 200
save_line_search_gif("rect_cover_norm.gif", generate_data_norm(20, IMG_SZ, IMG_SZ//12))


IMG_SZ = 600
TILE_SIZE = 100
save_line_search_gif("rect_cover_small.gif", generate_data_norm(50, IMG_SZ, IMG_SZ//20))
