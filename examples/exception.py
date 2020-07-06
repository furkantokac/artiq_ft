from artiq.experiment import *
from artiq.language.core import TerminationRequested

class ExceptionDemo(EnvExperiment):
    def build(self):
        self.setattr_device("core")
        self.setattr_device("led0")

    def foo(self):
        print("raise error")
        raise Exception

    def termination(self):
        raise TerminationRequested

    @rpc
    def remote(self):
        raise Exception

    @kernel
    def run(self):
        self.core.reset()
        try:
            try:
                self.foo()
            except ValueError as e:
                print("should not trigger this")
        except:
            print("catch all")

        try:
            self.remote()
        except:
            print("Error!")

        print("Uncaught error at last")
        self.termination()

