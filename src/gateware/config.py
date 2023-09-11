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
        f.write(cpu_interface.get_rust_cfg(
            soc.get_csr_regions(), soc.get_constants()))
