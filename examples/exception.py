from artiq.experiment import *

class ExceptionDemo(EnvExperiment):
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")

    @kernel
    def run(self):
        self.core.reset()
        print("OK!")
        try:
            try:
                raise Exception
            except ValueError as e:
                print("re-raise")
                raise e
        except:
            print("error")

