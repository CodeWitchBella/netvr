using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class StartInput: IEquatable<StartInput>, ICloneable {
        public ulong instance;
        public ulong session;
        public string data_directory;

        public StartInput(ulong _instance, ulong _session, string _data_directory) {
            instance = _instance;
            session = _session;
            if (_data_directory == null) throw new ArgumentNullException(nameof(_data_directory));
            data_directory = _data_directory;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_u64(instance);
            serializer.serialize_u64(session);
            serializer.serialize_str(data_directory);
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

        public static StartInput Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            StartInput obj = new StartInput(
            	deserializer.deserialize_u64(),
            	deserializer.deserialize_u64(),
            	deserializer.deserialize_str());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static StartInput BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static StartInput BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            StartInput value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is StartInput other && Equals(other);

        public static bool operator ==(StartInput left, StartInput right) => Equals(left, right);

        public static bool operator !=(StartInput left, StartInput right) => !Equals(left, right);

        public bool Equals(StartInput other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!instance.Equals(other.instance)) return false;
            if (!session.Equals(other.session)) return false;
            if (!data_directory.Equals(other.data_directory)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + instance.GetHashCode();
                value = 31 * value + session.GetHashCode();
                value = 31 * value + data_directory.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public StartInput Clone() => (StartInput)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
