import os
from PIL import Image, ImageDraw, ImageFont
import PIL
import numpy as np

IMG_SZ = 400
regular = (0,0,0,int(255*0.2))
selected = (0,0,0,int(255))

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


def draw_line(imgdraw, pos, vline, color=selected, width=1, rotate=0):
    bottom, top = vline
    left = right = pos
    if rotate:
        left, right, bottom, top = bottom, top, left, right
    imgdraw.line([(left, bottom), (right, top)], fill=color, width=width)


def bounding_box(boxes):
    x1s, y1s, x2s, y2s = boxes.T
    return np.array([
        x1s.min(),
        y1s.min(),
        x2s.max(),
        y2s.max(),
    ])


def generate_recursively(base_img, base_lines_img, boxes, rotate=0, boxwidth=8):
    my_img = base_img#Image.new('RGBA', (IMG_SZ, IMG_SZ), (255,255,255,0))
    # my_img = Image.alpha_composite(base_img, my_img)
    fnt = ImageFont.truetype(os.path.join(os.path.dirname(__file__),"07558_CenturyGothic.ttf"), 35)
    draw = ImageDraw.Draw(my_img)
    bbox = bounding_box(boxes)
    draw_cbox(draw, bbox, color=(220,0,220,255), width=boxwidth, rotate=rotate)
    lines_img = base_lines_img.copy()
    yield Image.alpha_composite(my_img, lines_img)
    GOAL_NODE_SIZE = 4
    MIN_BIN_SIZE = 6
    if len(boxes) <= MIN_BIN_SIZE:
        return
    lines_draw = ImageDraw.Draw(lines_img)

    sidxs = boxes[:,0].argsort()
    lefts = boxes[:,0]
    child_sizes = (len(boxes) + GOAL_NODE_SIZE - 1) // GOAL_NODE_SIZE
    left_idxs = np.concatenate([np.arange(len(boxes) // child_sizes) * child_sizes, np.array([len(boxes)])],axis=0)

    for i in range(0,len(left_idxs)-1):
        draw_line(lines_draw, pos=boxes[sidxs[left_idxs[i]]][0], vline=(bbox[1], bbox[3]),color=(0,255,0,255),rotate=rotate,width=max(1,(2*boxwidth//3)))
        yield Image.alpha_composite(my_img, lines_img)

    yield Image.alpha_composite(my_img, lines_img)

    for i in range(len(left_idxs)-1):
        lidx, ridx = left_idxs[i:i+2]
        child_boxes = boxes[sidxs[lidx:ridx]]
        rotated_children = np.stack([child_boxes[:,1],child_boxes[:,0],child_boxes[:,3],child_boxes[:,2]],axis=1)

        for frame in generate_recursively(my_img, lines_img, rotated_children, rotate=1-rotate, boxwidth=max(1,(2*boxwidth)//4)):
            yield frame
    yield Image.alpha_composite(my_img, lines_img)
    
    # left_idxs.pop()
    # complete the last sector
    # left_idxs.append(len(boxes))



def generate_recursive_tree_sort(boxes):
    base = Image.new('RGBA', (IMG_SZ, IMG_SZ), (255,255,255,255))
    lines_img = Image.new('RGBA', (IMG_SZ, IMG_SZ), (255,255,255,0), )
    draw = ImageDraw.Draw(base)
    for b in boxes:
        draw_cbox(draw, b, color='black',width=1,rotate=0)
    for frame in generate_recursively(base, lines_img, boxes):
        yield frame
    # pause before ending
    for i in range(30):
        yield base


def dup_images_for_slowness(img_gen):
    for img in img_gen:
        for i in range(10):
            yield img

def save_line_search_gif(fname, boxes):
    img_generator = dup_images_for_slowness(generate_recursive_tree_sort(boxes))
    img1 = next(img_generator)

    img1.save(f'docs/{fname}',
                save_all=True, append_images=img_generator, optimize=True, duration=6, loop=0)


save_line_search_gif("tile_sort.gif", generate_data(70, IMG_SZ, IMG_SZ//8))
IMG_SZ = 600
save_line_search_gif("tile_sort_norm.gif", generate_data_norm(100, IMG_SZ, IMG_SZ//12))

IMG_SZ = 1000
save_line_search_gif("tile_sort_large.gif", generate_data(200, IMG_SZ, IMG_SZ//16))
