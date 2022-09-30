
import matplotlib.pyplot as plt
import numpy as np
import sys

# Very basic script to use matplotlib to plot raw data saved from psrec
# Designed (currently) to only use bare minimal libs (python + matplotlib)

# TODO: Make this more pythonic (PEP8), and robust (error handling)

# TODO: we could do the pre-processing in the Rust implementation before saving to be more efficient,
#       although we'd have to save it as comment metadata in primitive file formats (CSV)...

def readDataValuesFromCSVFile(filename):
    timeValues = []
    cpuValues = []
    rssValues = []

    timeUnit = "s"
    rssUnit = "mb"

    fData = open(filename, "r")
    for line in fData:
        if len(line) == 0:
            continue
        if line[0] == '#':
            continue
        # is this a good idea? Might be better to error...
        if not ',' in line:
            continue

        items = line.split(',')
        time = items[0]
        cpu = items[1]
        rss = items[2]

        rssVal = float(rss) / 1024.0 / 1024.0
        if rssVal > 4000.0:
            rssUnit = "gb"

        timeValues.append(float(time))
        cpuValues.append(float(cpu))
        rssValues.append(rssVal)
    
    # See what last time was, and if > 5 mins, use minutes instead of seconds as the units
    if len(timeValues) > 0:
        if timeValues[-1] > (60.0 * 5.0):
            timeUnit = "m"
    
    if rssUnit == "gb":
        # Resize numbers to GB size
        # TODO: could use numpy for this
        for i in range(len(rssValues)):
            rssValues[i] /= 1024.0
    
    values = {'tv':timeValues, 'cv': cpuValues, 'rv': rssValues, 'tu':timeUnit, 'ru':rssUnit}
    return values

def generateBasicPlot(dataValues):
    fig, ax = plt.subplots(1, 1)

    fig.tight_layout()
    fig.set_figwidth(15)
    fig.set_figheight(8)

    # Colours for axis labels (which are darker than the plot colours), so we don't need to bother
    # with legends...
    CPU_AXIS_COLOUR = "#0000bf"
    RSS_AXIS_COLOUR = "#bf0000"

    timeValues = dataValues['tv']

    ax2 = ax.twinx()
    ax.set_title('Process recording (CPU usage and RSS memory usage)')
    xLabel = "Time elapsed ({})".format("Minutes" if dataValues['tu'] == "m" else "Seconds")
    ax.set_xlabel(xLabel)
    ax.plot(timeValues, dataValues['cv'], color='blue')
    ax2.plot(timeValues, dataValues['rv'], color='red')
    ax.set_ylabel('CPU usage (normalised %)', color=CPU_AXIS_COLOUR)
    ax2.set_ylabel('Memory RSS (MB)', color=RSS_AXIS_COLOUR)
#    ax.legend(['CPU Usage'])
#    ax2.legend(['RSS'])
    
    ax.yaxis.grid(color='lightgray')

    fig.tight_layout()

    ax.set_ylim(ymin=0)
    ax2.set_ylim(ymin=0)
    plt.show()

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Error: no input file provided.")
        exit(-1)
    
    inputFile = sys.argv[1]

    dataValues = readDataValuesFromCSVFile(inputFile)
    generateBasicPlot(dataValues)
