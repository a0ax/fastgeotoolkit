import csv
import json
import xml.etree.ElementTree as ET
from xml.dom import minidom

# Open and read the CSV file
with open("data/test.csv", "r", encoding="utf-8") as csv_file:
    csv_reader = csv.reader(csv_file)
    
    for row in csv_reader:
        if not row:
            continue
            
        # Extract the route ID and the JSON string column
        route_id = row[0].strip().replace('"', '')
        json_coords_str = row[-1]
        
        try:
            # Parse the inner JSON string into a list of coordinates
            coordinates = json.loads(json_coords_str)
            
            # 1. Create a fresh GPX structure for this specific route
            gpx = ET.Element("gpx", version="1.1", creator="Python CSV to GPX", xmlns="http://topografix.com")
            trk = ET.SubElement(gpx, "trk")
            name = ET.SubElement(trk, "name")
            name.text = f"Route {route_id}"
            
            trkseg = ET.SubElement(trk, "trkseg")
            
            # 2. Add coordinates to this route's segment (swapping lon/lat)
            for coord in coordinates:
                lon = coord[0]
                lat = coord[1]
                ET.SubElement(trkseg, "trkpt", lat=str(lat), lon=str(lon))
                
            # 3. Format the XML nicely
            xml_string = ET.tostring(gpx, encoding="utf-8")
            pretty_xml = minidom.parseString(xml_string).toprettyxml(indent="  ")
            
            # 4. Save to a unique file named after the Route ID
            filename = f"data/gpx/Route_{route_id}.gpx"
            with open(filename, "w", encoding="utf-8") as gpx_file:
                gpx_file.write(pretty_xml)
                
            print(f"Created file: {filename}")
                
        except (json.JSONDecodeError, IndexError) as e:
            print(f"Skipping row for Route {route_id} due to formatting issue: {e}")

print("All files processed!")
