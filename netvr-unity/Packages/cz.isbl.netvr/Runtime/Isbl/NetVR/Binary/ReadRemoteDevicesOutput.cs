using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class ReadRemoteDevicesOutput: IEquatable<ReadRemoteDevicesOutput>, ICloneable {
        public Serde.ValueArray<RemoteDevice> devices;

        public ReadRemoteDevicesOutput(Serde.ValueArray<RemoteDevice> _devices) {
            if (_devices == null) throw new ArgumentNullException(nameof(_devices));
            devices = _devices;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            TraitHelpers.serialize_vector_RemoteDevice(devices, serializer);
            serializer.decrease_container_depth();
        }

        public int BincodeSerialize(byte[] outputBuffer) => BincodeSerialize(new ArraySegment<byte>(outputBuffer));

        public int BincodeSerialize(ArraySegment<byte> outputBuffer) {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer(outputBuffer);
            Serialize(serializer);
            return serializer.get_buffer_offset();
        }

        public byte[] BincodeSerialize()  {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer();
            Serialize(serializer);
            return serializer.get_bytes();
        }

        public static ReadRemoteDevicesOutput Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            ReadRemoteDevicesOutput obj = new ReadRemoteDevicesOutput(
            	TraitHelpers.deserialize_vector_RemoteDevice(deserializer));
            deserializer.decrease_container_depth();
            return obj;
        }

        public static ReadRemoteDevicesOutput BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static ReadRemoteDevicesOutput BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            ReadRemoteDevicesOutput value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is ReadRemoteDevicesOutput other && Equals(other);

        public static bool operator ==(ReadRemoteDevicesOutput left, ReadRemoteDevicesOutput right) => Equals(left, right);

        public static bool operator !=(ReadRemoteDevicesOutput left, ReadRemoteDevicesOutput right) => !Equals(left, right);

        public bool Equals(ReadRemoteDevicesOutput other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!devices.Equals(other.devices)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + devices.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public ReadRemoteDevicesOutput Clone() => (ReadRemoteDevicesOutput)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
