package com.security.smith.client.message;

public class Heartbeat {
    private String filter;
    private String block;
    private String limit;
    private String patch;
    private String class_filter_version;
    private int discard_count;

    public String getFilter() {
        return filter;
    }

    public void setFilter(String filter) {
        this.filter = filter;
    }

    public String getBlock() {
        return block;
    }

    public void setBlock(String block) {
        this.block = block;
    }

    public String getLimit() {
        return limit;
    }

    public void setLimit(String limit) {
        this.limit = limit;
    }

    public String getPatch() {
        return patch;
    }

    public void setPatch(String patch) {
        this.patch = patch;
    }

    public String getClassFilterVersion() {
        return class_filter_version;
    }

    public void setClassFilterVersion(String classFilterVersion) {
        this.class_filter_version = classFilterVersion;
    }

    public synchronized int  getDiscardCount() {
        return discard_count;
    }

    public synchronized void setDiscardCount(int discard_count) {
        this.discard_count = discard_count;
    }
}
