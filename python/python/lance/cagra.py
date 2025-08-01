
import cupy as cp
from cuvs.neighbors import cagra
import time
import pickle
from datetime import datetime

def build_cagra_index(data, ids, cagra_params):
    print("Cagra input params: ", cagra_params)

    print("starting move of data to gpu time")
    startCpData = time.time()
    cp_data = cp.array(data, dtype=cp.float32)
    endCpData = time.time()
    print(f"\nTime to move data to gpu {endCpData - startCpData:.2f} seconds\n")

    print("starting build index")
    from datetime import datetime
    current_datetime = datetime.now()
    print(current_datetime.time())
    startBuildIndex = time.time()
    index = cagra.build(cagra.IndexParams(build_algo=cagra_params["algo"],
                                          intermediate_graph_degree=int(cagra_params["intermediate_graph_degree"]),
                                          graph_degree=int(cagra_params["graph_degree"])),
                                          cp_data)
    current_datetime = datetime.now()
    print(current_datetime.time())
    endBuildIndex = time.time()
    print(f"\nTime to build index {endBuildIndex - startBuildIndex:.2f} seconds\n")

    cagra.save("/workspace/cagra_index.bin", index)
    with open("/workspace/cagra_ids.pkl", "wb") as file:
        pickle.dump(ids, file)