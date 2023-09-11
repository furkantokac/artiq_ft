from misoc.integration import cpu_interface

def write_csr_file(soc, filename):
    with open(filename, "w") as f:
        f.write(cpu_interface.get_csr_rust(
            soc.get_csr_regions(), soc.get_csr_groups(), soc.get_constants()))

def write_mem_file(soc, filename):
    with open(filename, "w") as f:
        f.write(cpu_interface.get_mem_rust(
            soc.get_memory_regions(), soc.get_memory_groups(), None))

def write_rustc_cfg_file(soc, filename):
    with open(filename, "w") as f:
        for name, origin, busword, obj in soc.get_csr_regions():
            f.write("has_{}\n".format(name.lower()))
        for name, value in soc.get_constants():
            if name.upper().startswith("CONFIG_"):
                if value is None:
                    f.write("{}\n".format(name.lower()[7:]))
                else:
                    f.write("{}=\"{}\"\n".format(name.lower()[7:], str(value)))
