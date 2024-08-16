#!/usr/bin/env python3

'''
 psrec
 Copyright 2022-2024 Peter Pearson.
 Licensed under the Apache License, Version 2.0 (the "License");
 You may not use this file except in compliance with the License.
 You may obtain a copy of the License at
 http://www.apache.org/licenses/LICENSE-2.0
 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
 ---------
'''

import argparse
import matplotlib as mpl
import matplotlib.pyplot as plt

# *Very* basic script to use matplotlib to plot raw data saved from psrec
# Designed (currently) to only use bare minimal libs (python + matplotlib)

# TODO: Make this more pythonic (PEP8), and robust (error handling)

# TODO: we could do the pre-processing in the Rust implementation before saving to be more efficient,
#       although we'd have to save it as comment metadata in primitive file formats (CSV)...

def readDataValuesFromCSVFile(filename):
    timeValues = []
    cpuValues = []
    rssValues = []
    # Note: these are optional, and might therefore be empty
    threadCountsValues = []
    fileDescriptorCountValues = []

    timeUnit = "s"
    rssUnit = "mb"

    maxCPUValue = 0.0
    cpuType = None
    systemThreads = None

    haveThreadCount = False
    haveOpenFDCount = False

    fData = open(filename, "r")
    for line in fData:
        if len(line) == 0:
            continue
        if line[0] == '#':
            # see if it's a 'metadata' field comment
            if len(line) >= 4 and line[1] == '@':
                # it should be a metadata item, starting after the '#@ ' string,
                # so try and interpret it...
                metadata = line[2:]
                metadata_items = [x.strip() for x in metadata.split(':')]
                if metadata_items[0] == "cputype":
                    if metadata_items[1] == "normalised":
                        cpuType = "normalised"
                    elif metadata_items[1] == "absolute":
                        cpuType = "absolute"
                    else:
                        print("Unexpected 'cputype' metadata value.")
                elif metadata_items[0] == "systhreads":
                    systemThreads = int(metadata_items[1])
            elif len(line) >= 5 and "CPU Usage" in line:
                # it's the description of the fields (which will always have 'CPU Usage' in the string),
                # so parse that to work out what additional fields might also be in the data
                dataFieldItems = line[2:].strip().split(",")
                if "Thread Count" in dataFieldItems:
                    haveThreadCount = True
                if "FD Count" in dataFieldItems:
                    haveOpenFDCount = True
            continue
        # is this a good idea? Might be better to error...
        if not ',' in line:
            continue

        items = line.split(',')
        time = items[0]
        cpu = items[1]
        rss = items[2]

        cpu = float(cpu)

        # keep track of the maximum cpu value, even if we know if the type is normalised or not, and if
        # we know the number of system threads from the metadata.
        if cpu > maxCPUValue:
            maxCPUValue = cpu

        rssVal = float(rss) / 1024.0 / 1024.0
        if rssVal > 4000.0:
            rssUnit = "gb"

        timeValues.append(float(time))
        cpuValues.append(cpu)
        rssValues.append(rssVal)

        if len(items) > 3:
            # we have additional values, so work out what they are
            if len(items) == 4:
                if haveThreadCount:
                    threadCountsValues.append(int(items[3]))
                elif haveOpenFDCount:
                    fileDescriptorCountValues.append(int(items[3]))
            elif len(items) == 5:
                # currently it should be both these in this order
                threadCountsValues.append(int(items[3]))
                fileDescriptorCountValues.append(int(items[4]))
  
    # if there were no valid values, exit out...
    if len(timeValues) == 0:
        return None
    
    # See what last time was, and if > 5 mins, use hours/minutes instead of seconds as the units
    if timeValues[-1] > (60.0 * 60.0 * 5.0):
        # Use hours
        timeUnit = "h"
        # Also resize the numbers
        for i in range(len(timeValues)):
            timeValues[i] /= 3600.0
    elif timeValues[-1] > (60.0 * 5.0):
        # Use minutes
        timeUnit = "m"
        # Also resize the numbers
        for i in range(len(timeValues)):
            timeValues[i] /= 60.0
    
    if rssUnit == "gb":
        # Resize numbers to GB size
        # TODO: could use numpy for this
        for i in range(len(rssValues)):
            rssValues[i] /= 1024.0
    
    # see if we need to guess what the cpu type was
    if cpuType is None:
        # we don't know, so try and guess...
        cpuType = "absolute" if maxCPUValue > 102.0 else "normalised"
        
    values = {'tv':timeValues, 'cv':cpuValues, 'rv':rssValues, 'tcv':threadCountsValues, 'fdcv':fileDescriptorCountValues,
              'tu':timeUnit, 'ru':rssUnit,
              'cpuType':cpuType, 'sysThreads':systemThreads, 'mcv':maxCPUValue}
    return values

def generateBasicCombinedPlot(dataValues, args):
    fig, ax1 = plt.subplots(1, 1)

    fig.tight_layout()
    fig.set_figwidth(15)
    fig.set_figheight(8)

    # Colours for axis labels (which are darker than the plot colours), so we don't need to bother
    # with legends...
    CPU_AXIS_COLOUR = "#0000bf"
    RSS_AXIS_COLOUR = "#bf0000"

    timeValues = dataValues['tv']

    ax2 = ax1.twinx()
    ax1.set_title('Process recording (CPU usage and RSS memory usage)')
    xLabel = "Time elapsed ({})".format("Minutes" if dataValues['tu'] == "m" else "Hours" if dataValues['tu'] == "h" else "Seconds")
    ax1.set_xlabel(xLabel)
    if args.areaPlot:
        ax1.fill_between(timeValues, dataValues['cv'], color='blue', alpha=0.6)
        ax2.fill_between(timeValues, dataValues['rv'], color='red', alpha=0.6)
    else:
        ax1.plot(timeValues, dataValues['cv'], color='blue')
        ax2.plot(timeValues, dataValues['rv'], color='red')

    isCPUDataAbsolute = dataValues['cpuType'] == "absolute"

    ax1.set_ylabel('CPU usage ({} %)'.format("absolute" if isCPUDataAbsolute else "normalised"), color=CPU_AXIS_COLOUR)
    ax1.get_yaxis().set_major_formatter(mpl.ticker.FuncFormatter(lambda x, p: format(int(x), ',')))

    rssYLabel = "Memory RSS ({})".format("MB" if dataValues['ru'] == "mb" else "GB")
    ax2.set_ylabel(rssYLabel, color=RSS_AXIS_COLOUR)
    ax2.get_yaxis().set_major_formatter(mpl.ticker.FuncFormatter(lambda x, p: format(int(x), ',')))
    
    ax1.yaxis.grid(color='lightgray')
    if args.verticalGridLines:
        ax1.xaxis.grid(color='lightgray')

    fig.tight_layout()

    if isCPUDataAbsolute:
        ax1.set_ylim(ymin=0, ymax=dataValues['mcv'])
    else:
        ax1.set_ylim(ymin=0, ymax=101.0)
    ax1.set_xlim(xmin=0, xmax=timeValues[-1])
    ax2.set_ylim(ymin=0, ymax=None)
    ax2.set_xlim(xmin=0, xmax=timeValues[-1])
    plt.show()

def generateBasicSeparatePlot(dataValues, args):
    haveThreadCounts = len(dataValues['tcv']) > 0 and not args.nothreadcountplot
    haveFDCounts = len(dataValues['fdcv']) > 0 and not args.nofiledescriptorplot

    numPlots = 2
    titleItems = []
    if args.nocpuplot:
        numPlots -= 1
    else:
        titleItems.append("CPU usage")
    if args.norssplot:
        numPlots -= 1
    else:
        titleItems.append("RSS memory usage")
    if haveThreadCounts:
        numPlots += 1
        titleItems.append("Thread count")
    if haveFDCounts:
        numPlots += 1
        titleItems.append("Open FD count")
    
    # there's no point plotting 0 items, and currently we don't support only plotting 1
    # either as axes is not a subscriptable array in that scenario, so the code below doesn't
    # support that currently.
    if numPlots <= 1:
        print("Error: psrec_gen_plot currently only supports plotting two or more plots at once.")
        exit(-1)

    fig, axes = plt.subplots(numPlots, 1)

    fig.tight_layout()
    fig.set_figwidth(15)
    fig.set_figheight(8)

    titleText = ", ".join(titleItems[:-1]) + " and " + titleItems[-1] if len(titleItems) > 1 else titleItems[-1]
    fig.suptitle('Process recording ({})'.format(titleText))

    timeValues = dataValues['tv']
    xLabel = "Time elapsed ({})".format("Minutes" if dataValues['tu'] == "m" else "Hours" if dataValues['tu'] == "h" else "Seconds")

    nextIndex = 0
    if not args.nocpuplot:
        axes[nextIndex].yaxis.grid(color='lightgray')
        if args.verticalgrid:
            axes[nextIndex].xaxis.grid(color='lightgray')
        if args.areaplot:
            axes[nextIndex].fill_between(timeValues, dataValues['cv'], color='blue', alpha=0.7)
        else:
            axes[nextIndex].plot(timeValues, dataValues['cv'], color='blue')
        
        isCPUDataAbsolute = dataValues['cpuType'] == "absolute"
        axes[nextIndex].set_xlabel(xLabel)
        axes[nextIndex].set_ylabel('CPU usage ({} %)'.format("absolute" if isCPUDataAbsolute else "normalised"))
        axes[nextIndex].get_yaxis().set_major_formatter(mpl.ticker.FuncFormatter(lambda x, p: format(int(x), ',')))
        if isCPUDataAbsolute:
            axes[nextIndex].set_ylim(ymin=0, ymax=dataValues['mcv'])
        else:
            axes[nextIndex].set_ylim(ymin=0, ymax=101.0)
        axes[nextIndex].set_xlim(xmin=0, xmax=timeValues[-1])
        nextIndex += 1
    
    if not args.norssplot:
        axes[nextIndex].yaxis.grid(color='lightgray')
        if args.verticalgrid:
            axes[nextIndex].xaxis.grid(color='lightgray')
        if args.areaplot:
            axes[nextIndex].fill_between(timeValues, dataValues['rv'], color='red', alpha=0.7)
        else:
            axes[nextIndex].plot(timeValues, dataValues['rv'], color='red')
        axes[nextIndex].set_xlabel(xLabel)

        rssYLabel = "Memory RSS ({})".format("MB" if dataValues['ru'] == "mb" else "GB")
        axes[nextIndex].set_ylabel(rssYLabel)
        axes[nextIndex].get_yaxis().set_major_formatter(mpl.ticker.FuncFormatter(lambda x, p: format(int(x), ',')))
        axes[nextIndex].set_ylim(ymin=0, ymax=None)
        axes[nextIndex].set_xlim(xmin=0, xmax=timeValues[-1])
        nextIndex += 1

    if haveThreadCounts:
        axes[nextIndex].yaxis.grid(color='lightgray')
        if args.verticalgrid:
            axes[nextIndex].xaxis.grid(color='lightgray')
        if args.areaplot:
            axes[nextIndex].fill_between(timeValues, dataValues['tcv'], color='green', alpha=0.7)
        else:
            axes[nextIndex].plot(timeValues, dataValues['tcv'], color='green')
        axes[nextIndex].set_xlabel(xLabel)

        axes[nextIndex].set_ylabel("Active Thread Count")
        axes[nextIndex].get_yaxis().set_major_formatter(mpl.ticker.FuncFormatter(lambda x, p: format(int(x), ',')))
        axes[nextIndex].set_ylim(ymin=0, ymax=None)
        axes[nextIndex].set_xlim(xmin=0, xmax=timeValues[-1])
        nextIndex += 1

    if haveFDCounts:
        axes[nextIndex].yaxis.grid(color='lightgray')
        if args.verticalgrid:
            axes[nextIndex].xaxis.grid(color='lightgray')
        if args.areaplot:
            axes[nextIndex].fill_between(timeValues, dataValues['fdcv'], color='gold', alpha=0.7)
        else:
            axes[nextIndex].plot(timeValues, dataValues['fdcv'], color='gold')
        axes[nextIndex].set_xlabel(xLabel)

        axes[nextIndex].set_ylabel("Open File Descriptor Count")
        axes[nextIndex].get_yaxis().set_major_formatter(mpl.ticker.FuncFormatter(lambda x, p: format(int(x), ',')))
        axes[nextIndex].set_ylim(ymin=0, ymax=None)
        axes[nextIndex].set_xlim(xmin=0, xmax=timeValues[-1])
    
    fig.tight_layout()

    plt.show()

if __name__ == '__main__':
    parser = argparse.ArgumentParser(
                    prog='psrec generate plot',
                    description='Draws a plot of the data the main psrec program recorded, using Python and matplotlib',)
    
    parser.add_argument("inputFile", help="The input filename containing the raw data recording to plot.")
    parser.add_argument("--combined", action='store_true', default=False, help="Plot the recorded values in a combined single plot.")
    parser.add_argument("--areaplot", action='store_true', default=False, help="Plot the values as solid areas, rather than line plots.")
    parser.add_argument("--verticalgrid", action='store_true', default=False, help="Draw vertical grid lines for the Time axis.")
    parser.add_argument("--nocpuplot", action='store_true', default=False, help="Don't add a plot for CPU usage.")
    parser.add_argument("--norssplot", action='store_true', default=False, help="Don't add a plot for RSS memory usage.")
    parser.add_argument("--nothreadcountplot", action='store_true', default=False, help="Don't add a plot for Thread count.")
    parser.add_argument("--nofiledescriptorplot", action='store_true', default=False, help="Don't add a plot for Open File Descriptor count.")

    args = parser.parse_args()

    dataValues = readDataValuesFromCSVFile(args.inputFile)

    if not dataValues:
        print("Error: No valid recording data was found in the file specified to be plotted.")
        exit(-1)

    if args.combined:
        generateBasicCombinedPlot(dataValues, args)
    else:
        generateBasicSeparatePlot(dataValues, args)
    