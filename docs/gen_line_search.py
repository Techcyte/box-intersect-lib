from PIL import Image, ImageDraw
import PIL
import numpy as np

from box_intersect_lib_py import find_intersecting_boxes, intersect_area

IMG_SZ = 400
regular = (0,0,0,int(255*0.2))
selected = (0,0,0,int(255))

def generate_data(num_boxes, region_size, max_box_size):
    assert max_box_size > 0 and num_boxes > 0 and region_size > 0
    assert region_size > max_box_size, "Region size must be larger than the max box size"
    box_positions = np.random.randint(low=0, high=region_size-max_box_size, size=(num_boxes, 2), dtype="int32")
    box_sizes = np.random.randint(low=1, high=max_box_size, size=(num_boxes, 2), dtype="int32")
    boxes = np.concatenate([box_positions, box_sizes], axis=1)
    return boxes

def draw_box(imgdraw, box, color=selected, width=1):
    x1, y1, x2, y2 = box
    imgdraw.rectangle(((x1, y1), (x1+x2, y1+y2)), outline=color, fill=None, width=width)


def draw_state(imgdraw, boxes):
    for box in boxes:
        draw_box(imgdraw)

def boxs_to_set(boxes):
    return {tuple(b) for b in boxes}


def generate_line_search():
    boxes = generate_data(40, IMG_SZ, IMG_SZ//4)

    adj_list = find_intersecting_boxes(boxes)

    base = Image.new('RGBA', (IMG_SZ, IMG_SZ), (255,255,255,255))
    basedraw = ImageDraw.Draw(base)
    for b in boxes:
        draw_box(basedraw, b)

    idxs = boxes[:,0].argsort()
    sorted_adj_list = list(np.array(adj_list,dtype=object)[idxs])
    sorted_boxes = boxes[idxs]
    for i1, b1 in enumerate(sorted_boxes):
        intersect_list = []
        for i2, b2 in zip(range(i1, len(boxes)), sorted_boxes[i1:]):
            frame = Image.new('RGBA', (IMG_SZ, IMG_SZ), (255,255,255,0))
            draw = ImageDraw.Draw(frame)

            draw.rectangle(((b1[0], 0), (b1[0]+b1[2], IMG_SZ)), outline=None, fill=(200,200,0,100))
            draw.line([(b1[0], 0), (b1[0], IMG_SZ)], fill=(200,200,0,255), width=4)
            draw.line([(b1[0]+b1[2], 0), (b1[0]+b1[2], IMG_SZ)], fill=(200,200,0,255), width=4)
            draw_box(draw, b1, color=selected, width=4)
            if i1 != i2:
                for b in intersect_list:
                    draw_box(draw, b, color="blue", width=4)
                if b1[0] + b1[2] < b2[0]:
                    # let last frame hold for awhile before restarting
                    for b in boxs_to_set(boxes[sorted_adj_list[i1]]) - boxs_to_set(intersect_list):
                        draw_box(draw, b, color="green", width=4)
                    frame = Image.alpha_composite(base, frame)
                    for i in range(5):
                        yield frame
                    break

                if intersect_area(b1,b2.reshape(1,-1)).item() != 0:
                    draw_box(draw, b2, color="blue", width=4)
                    intersect_list.append(b2)
                else:
                    draw_box(draw, b2, color="red", width=4)

            frame = Image.alpha_composite(base, frame)
            yield frame


def dup_images_for_slowness(img_gen):
    for img in img_gen:
        for i in range(10):
            yield img

def save_line_search_gif():
    img_generator = dup_images_for_slowness(generate_line_search())
    img1 = next(img_generator)

    img1.save('docs/line_search.gif',
                save_all=True, append_images=img_generator, optimize=True, duration=6, loop=0)

save_line_search_gif()