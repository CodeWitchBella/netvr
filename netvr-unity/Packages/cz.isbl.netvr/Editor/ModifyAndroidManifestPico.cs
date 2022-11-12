using System;
using System.IO;
using System.Text;
using System.Xml;
using UnityEditor.Build.Reporting;
using UnityEditor.XR.OpenXR.Features;
using UnityEngine;

namespace Isbl.PicoOpenXR.PicoOpenXRSupport.Editor
{
    internal class ModifyAndroidManifestPico : OpenXRFeatureBuildHooks
    {
        public override int callbackOrder => 1;

        public override Type featureType => typeof(Isbl.PicoOpenXR.PicoOpenXRSupport.PicoOpenXRFeature);

        protected override void OnPreprocessBuildExt(BuildReport report)
        {
        }

        protected override void OnPostGenerateGradleAndroidProjectExt(string path)
        {
            var androidManifest = new AndroidManifest(GetManifestPath(path));
            androidManifest.AddPicoMetaData();
            androidManifest.Save();
        }

        protected override void OnPostprocessBuildExt(BuildReport report)
        {
        }

        private string _manifestFilePath;

        private string GetManifestPath(string basePath)
        {
            if (!string.IsNullOrEmpty(_manifestFilePath)) return _manifestFilePath;

            var pathBuilder = new StringBuilder(basePath);
            pathBuilder.Append(Path.DirectorySeparatorChar).Append("src");
            pathBuilder.Append(Path.DirectorySeparatorChar).Append("main");
            pathBuilder.Append(Path.DirectorySeparatorChar).Append("AndroidManifest.xml");
            _manifestFilePath = pathBuilder.ToString();

            return _manifestFilePath;
        }

        private class AndroidXmlDocument : XmlDocument
        {
            private string m_Path;
            protected XmlNamespaceManager nsMgr;
            public readonly string AndroidXmlNamespace = "http://schemas.android.com/apk/res/android";

            public AndroidXmlDocument(string path)
            {
                m_Path = path;
                using (var reader = new XmlTextReader(m_Path))
                {
                    reader.Read();
                    Load(reader);
                }

                nsMgr = new XmlNamespaceManager(NameTable);
                nsMgr.AddNamespace("android", AndroidXmlNamespace);
            }

            public string Save()
            {
                return SaveAs(m_Path);
            }

            public string SaveAs(string path)
            {
                using (var writer = new XmlTextWriter(path, new UTF8Encoding(false)))
                {
                    writer.Formatting = Formatting.Indented;
                    Save(writer);
                }

                return path;
            }
        }

        private class AndroidManifest : AndroidXmlDocument
        {
            private readonly XmlElement _manifestElement;

            public AndroidManifest(string path) : base(path)
            {
                _manifestElement = SelectSingleNode("/manifest") as XmlElement;
            }

            private bool HasAttributeWithValue(XmlNode node, string attributeName, string attributeValue)
            {
                foreach (XmlAttribute attrib in node.Attributes)
                {
                    if (attrib.NamespaceURI == AndroidXmlNamespace && attrib.Name == attributeName && attrib.Value == attributeValue) return true;
                }
                return false;
            }

            public void AddPicoMetaData()
            {
                // Get all child nodes that match the tag and see if value already exists
                var xmlNodeList = _manifestElement.SelectNodes("meta-data");
                foreach (XmlNode node in xmlNodeList)
                {
                    if (node is XmlElement element && HasAttributeWithValue(element, "name", "pvr.app.type"))
                    {
                        element.SetAttribute("value", AndroidXmlNamespace, "vr");
                        return;
                    }
                }

                XmlElement newElement = CreateElement("meta-data");
                newElement.SetAttribute("name", AndroidXmlNamespace, "pvr.app.type");
                newElement.SetAttribute("value", AndroidXmlNamespace, "vr");
                _manifestElement.AppendChild(newElement);
            }
        }
    }
}
