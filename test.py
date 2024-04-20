import clipppy
off = clipppy.Offseter(0.1, -1, 1, "square")
off.add_point(0,0)
off.add_point(5,0)
off.add_point(5,5)
off.add_point(0,5)
off.add_point(0,0)
res = off.offset_shape()
start = res.closest_start()
#res.fix_direction(start, False)
#res.first_point_delta(start)
res.reconstruct(start, lambda *args: print(args))
