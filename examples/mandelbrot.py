import sys

from artiq.experiment import *


class Mandelbrot(EnvExperiment):
    """Mandelbrot set demo"""

    def build(self):
        self.setattr_device("core")

    @rpc(flags={"async"})
    def col(self, i):
        sys.stdout.write(" .,-:;i+hHM$*#@ "[i])

    @rpc(flags={"async"})
    def row(self):
        print("")

    @rpc(flags={"async"})
    def prt(self, x):
        print(x)


    # based on: http://warp.povusers.org/MandScripts/python.html
    @kernel
    def run(self):
        minX = -2.0
        maxX = 1.0
        width = 78
        height = 36
        aspectRatio = 2

        yScale = (maxX-minX)*(height/width)*aspectRatio

        for y in range(height):
            for x in range(width):
                c_r = minX+x*(maxX-minX)/width
                c_i = y*yScale/height-yScale/2
                z_r = c_r
                z_i = c_i
                i = 0
                for i in range(16):
                    if z_r*z_r + z_i*z_i > 4:
                        break
                    new_z_r = (z_r*z_r)-(z_i*z_i) + c_r
                    z_i = 2*z_r*z_i + c_i
                    z_r = new_z_r
                self.col(i)
            self.row()
