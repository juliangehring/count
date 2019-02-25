{
    counts[$0]+=1
} END {
    PROCINFO["sorted_in"] = "@val_num_desc"
    for(key in counts) {
        print key, counts[key]
    }
}
