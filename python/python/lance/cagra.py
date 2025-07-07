
import cupy as cp
from cuvs.neighbors import cagra
import time

def build_cagra_index(data, cagra_params):
    print("python function")
    print(cagra_params)

    print("starting cp data build time")
    startCpData = time.time()
    cp_inner_data = [cp.array(d, dtype=cp.float32) for d in data]
    cp_data = cp.array(cp_inner_data)
    endCpData = time.time()
    print(f"\nTime to move data to gpu {endCpData - startCpData:.2f} seconds\n")

    print("starting build index")
    startBuildIndex = time.time()
    index = cagra.build(cagra.IndexParams(build_algo=cagra_params["algo"],  
                                          intermediate_graph_degree=int(cagra_params["intermediate_graph_degree"]),
                                          graph_degree=int(cagra_params["graph_degree"])), 
                                          cp_data)
    endBuildIndex = time.time()
    print(f"\nTime to build index {endBuildIndex - startBuildIndex:.2f} seconds\n")

    cagra.save("/workspace/cagra_index.bin", index)

def search_cagra(queries, path_to_index, k):
    # can add search params later
    print(queries)
    print("in search")